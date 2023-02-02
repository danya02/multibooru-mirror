use model::record::BooruRecord;

use crate::{
    persistence::{PersistenceError, PersistenceResult},
    Persistence, PersistenceSender,
};

/// A persistence backend that works by sending a copy of the incoming records to two other backends.
pub struct Duplicater<A, B, SenderA, SenderB>
where
    SenderA: PersistenceSender,
    SenderB: PersistenceSender,
    A: Persistence<Sender = SenderA>,
    B: Persistence<Sender = SenderB>,
{
    /// The first backend.
    first: A,
    /// The second backend.
    second: B,
    /// Whether the backend is shutting down, and no new records should be accepted.
    shutting_down_sender: tokio::sync::watch::Sender<bool>,
}

impl<
        SenderA: PersistenceSender,
        SenderB: PersistenceSender,
        A: Persistence<Sender = SenderA>,
        B: Persistence<Sender = SenderB>,
    > Duplicater<A, B, SenderA, SenderB>
where
    A: Persistence<Sender = SenderA>,
    B: Persistence<Sender = SenderB>,
{
    /// Create a new `Duplicater` backend.
    pub fn new(first: A, second: B) -> Self {
        let (shutting_down_sender, _) = tokio::sync::watch::channel(false);
        Self {
            first,
            second,
            shutting_down_sender,
        }
    }
}

#[async_trait::async_trait]
impl<A, B, SenderA, SenderB> Persistence for Duplicater<A, B, SenderA, SenderB>
where
    SenderA: PersistenceSender,
    SenderB: PersistenceSender,
    A: Persistence<Sender = SenderA> + Send,
    B: Persistence<Sender = SenderB> + Send,
{
    type Sender = DuplicaterSender<SenderA, SenderB>;
    async fn init(&mut self) {
        self.first.init().await;
        self.second.init().await;
    }

    fn get_sender(&self) -> DuplicaterSender<SenderA, SenderB> {
        DuplicaterSender {
            first: self.first.get_sender(),
            second: self.second.get_sender(),
            shutting_down_receiver: self.shutting_down_sender.subscribe(),
        }
    }

    async fn shutdown(&mut self) {
        self.shutting_down_sender.send_replace(true);
        tokio::join!(self.first.shutdown(), self.second.shutdown(),);
    }
}

pub struct DuplicaterSender<A: PersistenceSender, B: PersistenceSender> {
    first: A,
    second: B,
    shutting_down_receiver: tokio::sync::watch::Receiver<bool>,
}

#[async_trait::async_trait]
impl<A: PersistenceSender, B: PersistenceSender> PersistenceSender for DuplicaterSender<A, B> {
    #[allow(clippy::async_yields_async)]
    async fn submit_and_join(
        &self,
        record: BooruRecord,
    ) -> tokio::sync::oneshot::Receiver<PersistenceResult> {
        if *self.shutting_down_receiver.borrow() {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let _ = tx.send(Err(PersistenceError::ShuttingDown));
            return rx;
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        let first = self.first.submit_and_join(record.clone());
        let second = self.second.submit_and_join(record);
        // TODO: This will await both the results, defeating the purpose of returning a receiver.
        // Do we really need to return a receiver?
        let first = first.await;
        let second = second.await;
        let result = match (first.await, second.await) {
            (Ok(Ok(())), Ok(Ok(()))) => Ok(()),
            (Ok(Err(e)), Ok(second_res)) => {
                tracing::warn!("First persistence backend failed: {e:?}");
                second_res
            }
            (Ok(Ok(())), Ok(Err(e))) => Err(e),
            (Err(_e), _) => Err(PersistenceError::DuplicaterSenderDropped),
            (_, Err(_e)) => Err(PersistenceError::DuplicaterSenderDropped),
        };
        tx.send(result).unwrap();
        rx
    }
}
