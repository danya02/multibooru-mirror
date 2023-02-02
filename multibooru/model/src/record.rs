use common::Snowflake;
use serde::{Deserialize, Serialize};

/// Module for the global record type.

/// A record of a booru state.
///
/// This type is the most generic representation of things that have been recorded in a booru:
/// a new post, a new comment, a post change...
///
/// This top-level type stores the distinction between different types of boorus.
/// Booru-specific data is stored in the inner type.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Record {
    /// This record's snowflake ID.
    pub id: Snowflake,
    /// The data of this record.
    pub data: BooruRecord,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "booru", content = "data")]
pub enum BooruRecord {
    /// A record from Danbooru (https://danbooru.donmai.us).
    Danbooru(danbooru::record::Record),
}

impl BooruRecord {
    /// Get the ID of the type of booru this is.
    /// This must not change between versions.
    pub fn type_id(&self) -> u32 {
        match self {
            BooruRecord::Danbooru(_) => 1,
        }
    }
    /// Get the type of entity this record is about.
    pub fn entity_type_id(&self) -> u32 {
        match self {
            BooruRecord::Danbooru(record) => record.get_entity().type_id(),
        }
    }

    /// Get the booru ID of the entity this record is about.
    pub fn entity_id(&self) -> i64 {
        match self {
            BooruRecord::Danbooru(record) => record.get_entity().booru_id(),
        }
    }
}

impl From<danbooru::record::Record> for BooruRecord {
    fn from(record: danbooru::record::Record) -> Self {
        BooruRecord::Danbooru(record)
    }
}
