use danbooru::record::{Record, TagState};
use persistence::PersistenceSender;

/// Module for scraping tags from Danbooru.

/// Task that will loop forever, getting new tags from Danbooru and submitting them to the persistence layer.
pub async fn new_tags(sender: impl PersistenceSender) {
    let client = reqwest::Client::new();
    // Set client header

    // First, get the last tag ID as a starting point.
    let response = client
        .get("https://danbooru.donmai.us/tags.json?limit=1")
        .header("User-Agent", common::USER_AGENT)
        .send()
        .await
        .expect("Network error while getting the last tag ID."); // TODO: retry this with exponential backoff
    let text = response
        .text()
        .await
        .expect("The response was not a valid string.");
    log::debug!("Got response from Danbooru: {:?}", text);

    let last_tag = serde_json::from_str::<Vec<danbooru::Tag>>(&text)
        .expect("The response was not a valid JSON array of tags.")
        .into_iter()
        .next()
        .expect("The response contained no tags???");

    let mut last_tag_id = last_tag.id;

    // Also submit the last tag to the persistence layer.
    sender
        .submit(
            Record::Tag {
                id: last_tag.id,
                state: TagState::Exists(last_tag),
            }
            .into(),
        )
        .await;

    // Also submit that the tag after the last tag doesn't exist.
    sender
        .submit(
            Record::Tag {
                id: last_tag_id + 1,
                state: TagState::MissingTemporarily,
            }
            .into(),
        )
        .await;

    loop {
        // Get the tags after the last tag.
        let tags = client
            .get(&format!(
                "https://danbooru.donmai.us/tags.json?page=a{last_tag_id}"
            ))
            .header("User-Agent", common::USER_AGENT)
            .send()
            .await
            .expect("Network error while getting new tags.") // TODO: retry this with exponential backoff
            .json::<Vec<danbooru::Tag>>()
            .await
            .expect("The response was not a valid JSON array of tags.");

        // Submit the tags to the persistence layer, and update the last tag ID.
        let tag_count = tags.len();
        for tag in tags {
            last_tag_id = last_tag_id.max(tag.id);
            sender
                .submit(
                    Record::Tag {
                        id: tag.id,
                        state: TagState::Exists(tag),
                    }
                    .into(),
                )
                .await;
        }

        // Also submit that the tag after the last tag doesn't exist.
        // But only if there were some tags to begin with.
        if tag_count > 0 {
            sender
                .submit(
                    Record::Tag {
                        id: last_tag_id + 1,
                        state: TagState::MissingTemporarily,
                    }
                    .into(),
                )
                .await;
        }

        // Wait a bit before getting the next batch of tags.
        log::debug!("Waiting for next tags...");
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
