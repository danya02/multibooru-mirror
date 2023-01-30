use danbooru::record::{PostState, Record};
use persistence::PersistenceSender;

/// Module for scraping posts from Danbooru.

/// Task that will loop forever, getting new posts from Danbooru and submitting them to the persistence layer.
pub async fn new_posts(sender: impl PersistenceSender) {
    let client = reqwest::Client::new();
    // Set client header

    // First, get the last post ID as a starting point.
    let response = client
        .get("https://danbooru.donmai.us/posts.json?limit=1")
        .header(
            "User-Agent",
            "multibooru-scraper/0.1.0, +https://github.com/danya02/multibooru",
        )
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

    let mut last_post_id = last_post.id;

    // Also submit the last post to the persistence layer.
    sender
        .submit(Record::Post{id: last_post.id, state: PostState::Exists(last_post)}.into())
        .await;

    loop {
        // Get the posts after the last post.
        let posts = client
            .get(&format!(
                "https://danbooru.donmai.us/posts.json?page=a{}",
                last_post_id
            ))
            .header(
                "User-Agent",
                "multibooru-scraper/0.1.0, +https://github.com/danya02/multibooru",
            )
            .send()
            .await
            .expect("Network error while getting new posts.") // TODO: retry this with exponential backoff
            .json::<Vec<danbooru::Post>>()
            .await
            .expect("The response was not a valid JSON array of posts.");

        // Submit the posts to the persistence layer, and update the last post ID.
        for post in posts {
            last_post_id = last_post_id.max(post.id);
            sender
                .submit(Record::Post{id: post.id, state: PostState::Exists(post)}.into())
                .await;
        }

        // Also submit that the post after the last post doesn't exist.
        sender
            .submit(Record::Post{id: last_post_id + 1, state: PostState::Missing}.into())
            .await;

        // Wait a bit before getting the next batch of posts.
        log::debug!("Waiting for next posts...");
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
