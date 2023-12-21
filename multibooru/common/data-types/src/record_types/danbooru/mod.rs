use serde::{Deserialize, Serialize};

use self::comment::CommentRecord;
pub mod comment;
#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub enum DanbooruRecord {
    /// This describes the state of a comment.
    Comment(CommentRecord),
}

pub enum DanbooruRecordType {
    Comment = 1,
}

impl DanbooruRecord {
    pub fn entity_type(&self) -> DanbooruRecordType {
        match self {
            DanbooruRecord::Comment(_) => DanbooruRecordType::Comment,
        }
    }

    pub fn entity_id(&self) -> u64 {
        match self {
            DanbooruRecord::Comment(record) => record.entity_id(),
        }
    }
}
