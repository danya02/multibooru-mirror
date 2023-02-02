use danbooru::record::{PostState, Record};
use media_assets::scraping::MediaDownloadSender;
use persistence::PersistenceSender;

/// Module for scraping posts from Danbooru.

/// Task that will loop forever, getting new posts from Danbooru and submitting them to the persistence layer.
pub async fn new_posts(sender: impl PersistenceSender, file_download_sender: MediaDownloadSender) {
    let client = reqwest::Client::new();
    // Set client header

    // First, get the last post ID as a starting point.
    let response = client
        .get("https://danbooru.donmai.us/posts.json?limit=1")
        .header("User-Agent", common::USER_AGENT)
        .send()
        .await
        .expect("Network error while getting the last post ID."); // TODO: retry this with exponential backoff
    let text = response
        .text()
        .await
        .expect("The response was not a valid string.");
    log::debug!("Got response from Danbooru: {:?}", text);

    let last_post = serde_json::from_str::<Vec<danbooru::Post>>(&text)
        .expect("The response was not a valid JSON array of posts.")
        .into_iter()
        .next()
        .expect("The response contained no posts???");

    // Also start downloading this post.
    let dl_handle = media_assets::scraping::enqueue_download(&last_post.url, &file_download_sender);
    let mut last_post_id = last_post.id;

    let result = dl_handle.await;
    tracing::info!("Last post download result: {result:?}");

    // Also submit the last post to the persistence layer.
    sender
        .submit(
            Record::Post {
                id: last_post.id,
                state: PostState::Exists(last_post),
            }
            .into(),
        )
        .await;
    // Also submit that the post after the last post doesn't exist.
    sender
        .submit(
            Record::Post {
                id: last_post_id + 1,
                state: PostState::MissingTemporarily,
            }
            .into(),
        )
        .await;

    loop {
        // Get the posts after the last post.
        let posts = client
            .get(&format!(
                "https://danbooru.donmai.us/posts.json?page=a{last_post_id}",
            ))
            .header("User-Agent", common::USER_AGENT)
            .send()
            .await
            .expect("Network error while getting new posts.") // TODO: retry this with exponential backoff
            .json::<Vec<danbooru::Post>>()
            .await
            .expect("The response was not a valid JSON array of posts.");

        // Submit the posts to the persistence layer, and update the last post ID.
        let posts_count = posts.len();
        let mut post_urls = vec![];
        for post in posts {
            last_post_id = last_post_id.max(post.id);
            post_urls.push((post.id, post.url.clone()));
            sender
                .submit(
                    Record::Post {
                        id: post.id,
                        state: PostState::Exists(post),
                    }
                    .into(),
                )
                .await;
        }

        // Also submit that the post after the last post doesn't exist.
        // But only if we actually got some new posts.
        if posts_count > 0 {
            sender
                .submit(
                    Record::Post {
                        id: last_post_id + 1,
                        state: PostState::MissingTemporarily,
                    }
                    .into(),
                )
                .await;
        }

        // Now we can download the media assets for the posts we just got.
        let mut dl_results = vec![];
        for (id, url) in post_urls {
            let dl_result =
                media_assets::scraping::enqueue_download(&url, &file_download_sender).await;
            dl_results.push((id, dl_result));
        }
        // Await all the downloads.
        for (post_id, dl_result) in dl_results {
            match dl_result.await {
                Ok(_dl_result) => {
                    tracing::debug!("Downloaded media for post {}.", post_id);
                }
                Err(dl_error) => {
                    tracing::error!(
                        "Error while downloading media for post {}: {:?}",
                        post_id,
                        dl_error
                    );
                }
            }
        }

        // Wait a bit before getting the next batch of posts.
        log::debug!("Waiting for next posts...");
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
