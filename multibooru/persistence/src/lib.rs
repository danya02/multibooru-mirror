mod backend;
mod persistence;
pub use crate::persistence::{Persistence, PersistenceSender};

/// Make the persistence object.
pub async fn make_persistence() -> impl Persistence {
    let first = backend::PileOfFiles::new("records".into());
    let second = backend::sqlite_latest::SqliteLatest::new().await;
    let join = backend::duplicater::Duplicater::new(first, second);
    join
}
