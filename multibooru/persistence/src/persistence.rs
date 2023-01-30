use model::record::BooruRecord;

/// The `Persistence` trait is implemented by record storing backends.
/// Submitting records to this object will store them in the underlying database.
#[async_trait::async_trait]
pub trait Persistence {
    type Sender: PersistenceSender;

    /// Initialize any background threads needed by this backend.
    /// This function should be called before any other function.
    fn init(&mut self);

    /// Get a sender for this backend.
    /// This function can be called multiple times,
    /// but will panic if called before `init`.
    ///
    /// The returned sender can be passed to threads to submit records.
    fn get_sender(&self) -> Self::Sender;

    /// Shut down the backend.
    /// No new records will be accepted (returning a `PersistenceError::ShuttingDown` error).
    /// All pending records will be recorded before this function returns.
    async fn shutdown(&mut self);
}

/// A `PersistenceSender` is a handle to a `Persistence` backend.
/// It can be passed to threads to submit records.
#[async_trait::async_trait]
pub trait PersistenceSender: Sync + Send {
    /// Submit a record to the backend.
    ///
    /// This function may return before the record is actually stored, and it may never be.
    /// If you need to check if the record was stored, use `submit_and_join`.
    async fn submit(&self, record: BooruRecord) {
        // I'm not sure if it's possible to write a more performant `submit` than `submit_and_join`,
        // so the default implementation is to just call `submit_and_join` and drop the receiver.
        let recv = self.submit_and_join(record).await;
        std::mem::drop(recv);
    }

    /// Submit a record to the backend, returning a `tokio::sync::oneshot::Receiver`.
    /// The receiver will receive a `PersistenceResult` when the given task gets executed.
    /// You can then wait on the receiver to check if the record was stored.
    ///
    /// To the backend developer: you must not `unwrap` sending the result to the receiver,
    /// because the receiver may have been dropped by the time you get to it.
    /// In particular, `submit` is implemented this way by default.
    async fn submit_and_join(
        &self,
        record: BooruRecord,
    ) -> tokio::sync::oneshot::Receiver<PersistenceResult>;
}

pub type PersistenceResult = Result<(), PersistenceError>;

#[derive(Debug, Clone)]
pub enum PersistenceError {
    /// The backend is shutting down.
    /// No new records will be accepted.
    ShuttingDown,
    /// Unknown error.
    Unknown(String),
}
