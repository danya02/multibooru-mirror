mod backend;
mod persistence;
pub use persistence::{Persistence, PersistenceSender};

/// Make the persistence object.
pub fn make_persistence() -> impl Persistence {
    backend::PileOfFiles::new("records".into())
}
