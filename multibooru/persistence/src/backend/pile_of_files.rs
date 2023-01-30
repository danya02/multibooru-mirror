use std::path::PathBuf;

use common::Snowflake;
use model::record::BooruRecord;
use tokio::io::AsyncWriteExt;

use crate::persistence::{Persistence, PersistenceError, PersistenceResult, PersistenceSender};

/// A persistence backend that stores each incoming record in a separate file.
/// This is the most basic backend, and is used for testing.

pub struct PileOfFiles {
    /// The directory where the files are stored.
    directory: PathBuf,
    /// Whether the backend is shutting down, and no new records should be accepted.
    shutting_down_sender: tokio::sync::watch::Sender<bool>,
}

impl PileOfFiles {
    /// Create a new `PileOfFiles` backend.
    pub fn new(directory: PathBuf) -> Self {
        let (shutting_down_sender, _) = tokio::sync::watch::channel(false);
        Self {
            directory,
            shutting_down_sender,
        }
    }
}

#[async_trait::async_trait]
impl Persistence for PileOfFiles {
    type Sender = PileOfFilesSender;
    fn init(&mut self) {
        // Since this backend doesn't use any background threads, there's nothing to do here.
        // You could even not call this,
        // but other backends should panic if this isn't called.
    }

    fn get_sender(&self) -> PileOfFilesSender {
        PileOfFilesSender {
            directory: self.directory.clone(),
            shutting_down_receiver: self.shutting_down_sender.subscribe(),
        }
    }

    async fn shutdown(&mut self) {
        self.shutting_down_sender.send_replace(true);
    }
}

pub struct PileOfFilesSender {
    directory: PathBuf,
    shutting_down_receiver: tokio::sync::watch::Receiver<bool>,
}

#[async_trait::async_trait]
impl PersistenceSender for PileOfFilesSender {
    #[allow(clippy::async_yields_async)]
    async fn submit_and_join(
        &self,
        record: BooruRecord,
    ) -> tokio::sync::oneshot::Receiver<PersistenceResult> {
        async fn inner(this: &PileOfFilesSender, record: BooruRecord) -> PersistenceResult {
            if *this.shutting_down_receiver.borrow() {
                Err(PersistenceError::ShuttingDown)
            } else {
                let snow = Snowflake::new();
                log::debug!("Writing record {record:?} as {snow}.json");
                let mut file = tokio::fs::File::create(this.directory.join(format!("{snow}.json")))
                    .await
                    .map_err(|e| {
                        PersistenceError::Unknown(format!("Failed to create file: {e}"))
                    })?;

                file.write_all(serde_json::to_string(&record).unwrap().as_bytes())
                    .await
                    .unwrap();
                Ok(())
            }
        }

        // Because this backend is synchronous, we create a channel and immediately send the result to it.
        let (sender, receiver) = tokio::sync::oneshot::channel();
        sender.send(inner(self, record).await).unwrap();
        receiver
    }
}
