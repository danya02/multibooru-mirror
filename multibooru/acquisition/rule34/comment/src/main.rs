use std::time::Duration;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicPublishArguments, QueueBindArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use data_types::{
    record::{Record, RecordData},
    record_types::rule34::{
        comment::{CommentRecord, CommentState},
        Rule34Record,
    },
    USER_AGENT,
};
use rand::Rng;
use reqwest::Proxy;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    println!("Started!");
    tracing_subscriber::fmt::init();

    let mut rng = rand::thread_rng();

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

    let channel = connection
        .open_channel(None)
        .await
        .expect("Failed to open channel");

    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    println!("Channel created!");

    channel
        .queue_bind(QueueBindArguments::new(
            "new-records",
            "amq.direct",
            "new-records",
        ))
        .await
        .unwrap();

    println!("Performed binding...");

    let mut greatest_id_sent = 0;
    loop {
        let duration = rng.gen_range(25.0..35.0);
        let duration = std::time::Duration::from_secs_f32(duration);
        tracing::trace!("Sleeping for {duration:?}");
        tokio::time::sleep(duration).await;

        // Perform a GET to the API
        let request = client
            .get("https://api.rule34.xxx/index.php?page=dapi&s=comment&q=index")
            .send()
            .await;
        let resp = match request {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while retrieving comments: {e}");
                continue;
            }
        };

        let text = match resp.text().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while parsing comments as text: {e}");
                continue;
            }
        };

        // Try parsing the text as XML
        let mut comments: Comments = match serde_xml_rs::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error while parsing comments XML: {e}");
                continue;
            }
        };
        comments.comments.sort_by(|a, b| a.id.cmp(&b.id));
        for comment in comments.comments {
            if comment.id > greatest_id_sent {
                // Parse the date format
                let date = NaiveDateTime::parse_from_str(&comment.created_at, "%Y-%m-%d %H:%M");
                let date = match date {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Error while parsing comment's date: {e}");
                        continue;
                    }
                };
                let date: DateTime<Utc> = date.and_utc();

                // Send the comment
                let data = RecordData::Rule34(Rule34Record::Comment(CommentRecord {
                    id: comment.id,
                    state: CommentState::Present {
                        post_id: comment.post_id,
                        author_id: Some(comment.creator_id),
                        author_name: comment.creator,
                        created_at: date,
                        text: comment.body,
                        score: None,
                        is_reported: None,
                    },
                }));

                let mut record = Record::new(data);
                record.id = record.id.with_low_bits(rng.gen());

                let json_data = serde_json::to_vec(&record);
                let json_data = match json_data {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Error while encoding comment to be sent: {e}");
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

                greatest_id_sent = comment.id;
            }
        }
    }
}

#[derive(Deserialize)]
struct Comments {
    #[serde(rename = "$value")]
    comments: Vec<Comment>,
}

#[derive(Deserialize)]
struct Comment {
    created_at: String,
    post_id: u64,
    body: String,
    creator: String,
    id: u64,
    creator_id: u64,
}
