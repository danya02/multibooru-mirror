use std::time::Duration;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicRejectArguments, Channel, QueueBindArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use data_types::record::Record;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    println!("Started!");
    tracing_subscriber::fmt::init();

    let database_path: String =
        std::env::var("DATABASE_PATH").expect("DATABASE_PATH should be provided");

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
        .queue_bind(QueueBindArguments::new("new-records", "amq.topic", ""))
        .await
        .unwrap();

    println!("Performed binding!");

    let args = BasicConsumeArguments::new("new-records", "imageboards.persistence-reader");

    println!("Connecting to database on {database_path:?}");
    // Ensure that there exists a file at that path.
    {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&database_path)
            .expect("Failed to ensure that database file exists");
    }

    let sqlite_conn = SqlitePool::connect(&format!("sqlite://{database_path}"))
        .await
        .expect("Failed to open SQLite database");

    sqlx::query("CREATE TABLE IF NOT EXISTS record(id INTEGER NOT NULL PRIMARY KEY, type_id INTEGER NOT NULL, booru_id INTEGER NOT NULL, entity_type INTEGER NOT NULL, entity_id INTEGER NOT NULL, data_json TEXT NOT NULL);").execute(&sqlite_conn).await.expect("Failed to create table of records");
    sqlx::query("CREATE INDEX IF NOT EXISTS record_type_id ON record(type_id)")
        .execute(&sqlite_conn)
        .await
        .expect("Failed to create index of records");
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS record_entity ON record(booru_id, entity_type, entity_id)",
    )
    .execute(&sqlite_conn)
    .await
    .expect("Failed to create index of records");

    let consumer = MessageConsumer { conn: sqlite_conn };

    channel.basic_consume(consumer, args).await.unwrap();

    println!("Connected to queue, waiting for messages...");
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Closing because of signal");
                channel.close().await.unwrap();
                connection.close().await.unwrap();
                return;
            }
            _ = interval.tick() => {
                if !channel.is_open() || !connection.is_open() {
                    println!("Closing because connection is now closed");
                    channel.close().await.unwrap();
                    connection.close().await.unwrap();
                    return;
                }
            }
        }
        tracing::trace!("still alive~");
    }
}

struct MessageConsumer {
    conn: SqlitePool,
}

#[async_trait::async_trait]
impl AsyncConsumer for MessageConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        // Try parsing the message.
        // If we fail, this needs to be dead-lettered as an invalid Record.
        let new_record: Record = match serde_json::from_slice(&content) {
            Ok(v) => v,
            Err(why) => {
                tracing::error!("Received invalid JSON: {}", why);
                channel
                    .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), false))
                    .await
                    .unwrap();
                return;
            }
        };

        // Save the record into the persistence.
        if let Err(why) = 
        sqlx::query("INSERT INTO record(id, type_id, booru_id, entity_type, entity_id, data_json) VALUES (?,?,?,?,?,?)")
        .bind(i64::from(new_record.id)).bind(new_record.data.type_id() as i64)
        .bind(new_record.data.booru_id() as i64)
        .bind(new_record.data.entity_type() as i64)
        .bind(new_record.data.entity_id() as i64)
        .bind(serde_json::to_string(&new_record.data).expect("Failed to serialize")).execute(&self.conn).await {
            tracing::error!("Failed to insert record: {}", why);
            channel
                .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), false))
                .await
                .unwrap();
            return;
        }

        tracing::debug!("Inserted new record: {new_record:?}");

        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();
    }
}
