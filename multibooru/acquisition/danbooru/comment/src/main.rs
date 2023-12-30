use std::time::{Duration, SystemTime};

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicPublishArguments, QueueBindArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use data_types::{
    record::{Record, RecordData},
    record_types::danbooru::{
        comment::{CommentRecord, CommentState},
        DanbooruRecord,
    },
    DateTime, USER_AGENT,
};
use rand::Rng;
use reqwest::{Client, Proxy};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    println!("Started!");
    tracing_subscriber::fmt::init();

    let proxy_url = std::env::var("PROXY_URL").expect("PROXY_URL must be provided");

    let client = reqwest::ClientBuilder::new()
        .proxy(Proxy::all(&proxy_url).expect("Failed to create proxy from URL"))
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build Reqwest client");

    let connection = Connection::open(&OpenConnectionArguments::new(
        &std::env::var("AMQP_SERVER").expect("AMQP_SERVER should be provided"),
        5672,
        &std::env::var("AMQP_USER").expect("AMQP_USER should be provided"),
        &std::env::var("AMQP_PASSWORD").expect("AMQP_PASSWORD should be provided"),
    ))
    .await
    .expect("Failed to connect to AMQP");
    println!("Connection established!");

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    let new_comments = tokio::spawn(run_new_comment_loop(connection.clone(), client.clone()));
    let deleted_comments =
        tokio::spawn(run_deleted_comment_loop(connection.clone(), client.clone()));

    tokio::select! {
        why = new_comments => {println!("New comment loop exited first: {why:?}"); return;}
        why = deleted_comments => {println!("Deleted comment loop exited first: {why:?}"); return;}
        _ = tokio::signal::ctrl_c() => {println!("Exiting from signal"); return;}
    }
}

async fn run_new_comment_loop(connection: Connection, client: Client) {
    let channel = connection
        .open_channel(None)
        .await
        .expect("Failed to open channel");

    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    println!("New comment channel created!");

    channel
        .queue_bind(QueueBindArguments::new(
            "new-records",
            "amq.direct",
            "new-records",
        ))
        .await
        .unwrap();

    println!("Performed binding for new comment channel");

    let mut latest_update_sent = DateTime::from(SystemTime::UNIX_EPOCH);
    let mut delay_secs = 30.0;

    let mut error_count = 0;
    let mut count_error = || {
        error_count += 1;
        tracing::error!("Error detected in new loop (current count: {error_count})");
        if error_count > 5 {
            panic!("Too many errors in new loop!")
        }
    };

    loop {
        let duration = rand::thread_rng().gen_range(delay_secs - 5.0..delay_secs + 5.0);
        let duration = std::time::Duration::from_secs_f32(duration);
        tracing::debug!("Sleeping for {duration:?}");
        tokio::time::sleep(duration).await;

        // Perform a GET to the API
        let request = client
            .get("https://danbooru.donmai.us/comments.json?group_by=comment&limit=100&search[order]=updated_at&search[is_deleted]=false")
            .send()
            .await;
        let resp = match request {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while retrieving comments: {e}");
                count_error();
                continue;
            }
        };

        let text = match resp.text().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while parsing comments as text: {e}");
                count_error();
                continue;
            }
        };

        // Try parsing the text as JSON
        let mut comments: Vec<FullComment> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while parsing comments JSON: {e}");
                count_error();
                continue;
            }
        };

        let mut published = 0;

        comments.retain(|v| v.updated_at > latest_update_sent);
        for comment in comments {
            assert_eq!(comment.is_deleted, false);
            let FullComment {
                id,
                created_at,
                post_id,
                creator_id,
                body,
                score,
                updated_at,
                updater_id,
                do_not_bump_post,
                is_sticky,
                ..
            } = comment;

            // Send the comment
            let data = RecordData::Danbooru(DanbooruRecord::Comment(CommentRecord {
                id: id,
                state: CommentState::Present {
                    post_id,
                    created_at,
                    updated_at,
                    creator_id,
                    updater_id,
                    body,
                    score,
                    do_not_bump_post,
                    is_sticky,
                },
            }));

            let mut record = Record::new(data);
            record.id = record.id.with_low_bits(rand::random());

            let json_data = serde_json::to_vec(&record);
            let json_data = match json_data {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Error while encoding comment to be sent: {e}");
                    count_error();
                    continue;
                }
            };
            channel
                .basic_publish(
                    BasicProperties::default(),
                    json_data,
                    BasicPublishArguments {
                        exchange: "amq.direct".to_string(),
                        routing_key: "new-records".to_string(),
                        mandatory: true,
                        immediate: false,
                    },
                )
                .await
                .unwrap();
            published += 1;

            latest_update_sent = latest_update_sent.max(comment.updated_at);
        }
        tracing::debug!("Published {published} new comments");
        tracing::debug!("Current latest update for new: {latest_update_sent}");

        // Update the delay:
        // if zero comments were received, increase the time (up to a maximum of 1800 seconds),
        // if more than one comment was received, decrease the time (down to a minimum of 10 seconds).
        if published == 0 {
            delay_secs += 5.0;
        }
        if published > 1 {
            delay_secs -= 5.0;
        }
        delay_secs = delay_secs.min(1800.0).max(10.0);
    }
}

async fn run_deleted_comment_loop(connection: Connection, client: Client) {
    let channel = connection
        .open_channel(None)
        .await
        .expect("Failed to open channel");

    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    println!("Deleted comment channel created!");

    channel
        .queue_bind(QueueBindArguments::new(
            "new-records",
            "amq.direct",
            "new-records",
        ))
        .await
        .unwrap();

    println!("Performed binding for deleted comment channel");

    let mut latest_datetime_sent = DateTime::from(SystemTime::UNIX_EPOCH);
    let mut delay_secs = 150.0; // Comments are deleted much rarer than they are created.

    let mut error_count = 0;
    let mut count_error = || {
        error_count += 1;
        tracing::error!("Error detected in deleted loop (current count: {error_count})");
        if error_count > 5 {
            panic!("Too many errors in deleted loop!")
        }
    };

    loop {
        let duration = rand::thread_rng().gen_range(delay_secs - 5.0..delay_secs + 5.0);
        let duration = std::time::Duration::from_secs_f32(duration);
        tracing::debug!("Sleeping for {duration:?}");
        tokio::time::sleep(duration).await;

        // Perform a GET to the API
        let request = client
            .get("https://danbooru.donmai.us/comments.json?group_by=comment&limit=100&search[order]=updated_at&search[is_deleted]=true&only=id,created_at,post_id,updated_at,is_deleted,creator[id],updater[id]")
            .send()
            .await;
        let resp = match request {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while retrieving comments: {e}");
                count_error();
                continue;
            }
        };

        let text = match resp.text().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while parsing comments as text: {e}");
                count_error();
                continue;
            }
        };

        // Try parsing the text as JSON
        let mut comments: Vec<DeletedComment> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while parsing comments JSON: {e}");
                count_error();
                continue;
            }
        };
        let mut published = 0;

        comments.retain(|v| v.updated_at > latest_datetime_sent);
        for comment in comments {
            assert_eq!(comment.is_deleted, true);
            let DeletedComment {
                id,
                created_at,
                post_id,
                creator: IdHolder { id: creator_id },
                updated_at,
                updater: IdHolder { id: updater_id },
                is_deleted,
            } = comment;
            assert_eq!(is_deleted, true);

            // Send the comment
            let data = RecordData::Danbooru(DanbooruRecord::Comment(CommentRecord {
                id: id,
                state: CommentState::Deleted {
                    post_id,
                    created_at,
                    updated_at,
                    creator_id,
                    updater_id,
                },
            }));

            let mut record = Record::new(data);
            record.id = record.id.with_low_bits(rand::random());

            let json_data = serde_json::to_vec(&record);
            let json_data = match json_data {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Error while encoding comment to be sent: {e}");
                    count_error();
                    continue;
                }
            };
            channel
                .basic_publish(
                    BasicProperties::default(),
                    json_data,
                    BasicPublishArguments {
                        exchange: "amq.direct".to_string(),
                        routing_key: "new-records".to_string(),
                        mandatory: true,
                        immediate: false,
                    },
                )
                .await
                .unwrap();
            published += 1;

            latest_datetime_sent = latest_datetime_sent.max(comment.updated_at);
        }

        tracing::debug!("Published {published} deleted comments");
        tracing::debug!("Current latest update for deleted: {latest_datetime_sent}");

        // Update the delay:
        // if zero comments were received, increase the time (up to a maximum of 1800 seconds),
        // if more than one comment was received, decrease the time (down to a minimum of 10 seconds).
        if published == 0 {
            delay_secs += 5.0;
        }
        if published > 1 {
            delay_secs -= 5.0;
        }
        delay_secs = delay_secs.min(1800.0).max(10.0);
    }
}

#[derive(Deserialize)]
struct FullComment {
    id: u64,
    created_at: DateTime,
    post_id: u64,
    creator_id: u64,
    body: String,
    score: i64,
    updated_at: DateTime,
    updater_id: u64,
    do_not_bump_post: bool,
    is_deleted: bool,
    is_sticky: bool,
}

#[derive(Deserialize)]
struct DeletedComment {
    id: u64,
    created_at: DateTime,
    post_id: u64,

    creator: IdHolder,
    updated_at: DateTime,
    updater: IdHolder,

    is_deleted: bool,
}

#[derive(Deserialize)]
struct IdHolder {
    id: u64,
}
