use serde::{Deserialize, Serialize};

use self::comment::CommentRecord;
pub mod comment;
#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub enum Rule34Record {
    /// This describes the state of a comment.
    Comment(CommentRecord),
}

pub enum Rule34EntityType {
    Comment = 1,
}

impl Rule34Record {
    pub fn entity_type(&self) -> Rule34EntityType {
        match self {
            Rule34Record::Comment(_) => Rule34EntityType::Comment,
        }
    }

    pub fn entity_id(&self) -> u64 {
        match self {
            Rule34Record::Comment(record) => record.entity_id(),
        }
    }
}
