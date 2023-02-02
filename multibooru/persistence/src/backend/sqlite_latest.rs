use model::record::BooruRecord;
use sqlx::SqlitePool;

use crate::{
    persistence::{PersistenceError, PersistenceResult},
    Persistence, PersistenceSender,
};

/// A backend that stores the latest state of the records in an SQLite database.
pub struct SqliteLatest {
    /// The connection to the database.
    connection: SqlitePool,
    /// Whether the backend is shutting down, and no new records should be accepted.
    shutting_down_sender: tokio::sync::watch::Sender<bool>,
}

impl SqliteLatest {
    /// Create a new `SqliteLatest` backend.
    pub async fn new() -> Self {
        let connection = SqlitePool::connect(std::env::var("DATABASE_URL").unwrap().as_str())
            .await
            .expect("Failed to open database connection");
        let (shutting_down_sender, _) = tokio::sync::watch::channel(false);
        Self {
            connection,
            shutting_down_sender,
        }
    }
}

#[async_trait::async_trait]
impl Persistence for SqliteLatest {
    type Sender = SqliteLatestSender;
    async fn init(&mut self) {
        // Nothing to do here: the database gets initialized in the constructor.
    }

    fn get_sender(&self) -> SqliteLatestSender {
        SqliteLatestSender {
            connection: self.connection.clone(),
            shutting_down_receiver: self.shutting_down_sender.subscribe(),
        }
    }

    async fn shutdown(&mut self) {
        self.shutting_down_sender.send(true).unwrap();
    }
}

/// A sender for the `SqliteLatest` backend.
pub struct SqliteLatestSender {
    /// The connection to the database.
    connection: SqlitePool,
    /// Whether the backend is shutting down, and no new records should be accepted.
    shutting_down_receiver: tokio::sync::watch::Receiver<bool>,
}

#[async_trait::async_trait]
impl PersistenceSender for SqliteLatestSender {
    async fn submit_and_join(
        &self,
        record: BooruRecord,
    ) -> tokio::sync::oneshot::Receiver<PersistenceResult> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let connection = self.connection.clone();
        let shutting_down_receiver = self.shutting_down_receiver.clone();
        tokio::spawn(async move {
            if shutting_down_receiver.borrow().clone() {
                sender.send(Err(PersistenceError::ShuttingDown)).unwrap();
                return;
            }
            let booru_id = record.type_id();
            let entity_type_id = record.entity_type_id();
            let entity_id = record.entity_id();
            let entity_data_json = serde_json::to_string(&record).unwrap();
            let result = sqlx::query!("
                INSERT INTO latest_state (booru_id, entity_type_id, entity_id, entity_data_json)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (booru_id, entity_type_id, entity_id) DO UPDATE SET entity_data_json = $4
            ",
                booru_id,
                entity_type_id,
                entity_id,
                entity_data_json,
        ).execute(&connection).await;
            sender
                .send(result
                    .map(|_| ())
                    .map_err(|e| PersistenceError::DatabaseError(e)))
                .unwrap();
        });
        receiver
    }
}
