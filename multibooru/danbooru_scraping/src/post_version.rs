use danbooru::record::Record;
use persistence::PersistenceSender;

/// Module for scraping post versions from Danbooru.

/// Task that will loop forever, getting new post versions from Danbooru and submitting them to the persistence layer.
pub async fn new_post_versions(sender: impl PersistenceSender) {
    let client = reqwest::Client::builder();
    let client = common::proxy_maker::with_proxy(client);
    let client = client.user_agent(common::USER_AGENT);
    let client = client.build().expect("Failed to build the HTTP client.");

    // First, get the last post version ID as a starting point.
    let response = client
        .get("https://danbooru.donmai.us/post_versions.json?limit=1")
        .send().await
        .expect("Network error while getting the last post version ID."); // TODO: retry this with exponential backoff
    let text = response
        .text().await
        .expect("The response was not a valid string.");
    tracing::debug!("Got response from Danbooru: {:?}", text);

    let last_post_version = serde_json::from_str::<Vec<danbooru::PostVersion>>(&text)
        .expect("The response was not a valid JSON array of post versions.")
        .into_iter()
        .next()
        .expect("The response contained no post versions???");

    let mut last_post_version_id = last_post_version.id;

    // Also submit the last post version to the persistence layer.
    sender
        .submit(
            Record::PostVersion {
                id: last_post_version.id,
                state: last_post_version,
            }
            .into(),
        ).await;

    loop {
        // Get the post versions after the last post version.
        let post_versions = client
            .get(&format!(
                "https://danbooru.donmai.us/post_versions.json?page=a{last_post_version_id}"
            ))
            .send().await
            .expect("Network error while getting the post versions after the last post version.") // TODO: retry this with exponential backoff
            .json::<Vec<danbooru::PostVersion>>().await.expect("The response was not a valid JSON array of post versions.");
        tracing::debug!("Got response from Danbooru: {:?}", post_versions);
        // Submit the post versions to the persistence layer.
        for post_version in post_versions {
            last_post_version_id = post_version.id;
            sender
                .submit(
                    Record::PostVersion {
                        id: post_version.id,
                        state: post_version,
                    }
                    .into(),
                ).await;
        }

        // TODO!!: Also queue the post with the post version's post ID to be scraped.

        // Wait a bit before getting the next batch of post versions.
        tracing::debug!("Waiting for next post versions...");
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        
    }
}