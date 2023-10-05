use serde::{Deserialize, Serialize};

use crate::{record_types::media::MediaRecord, Snowflake};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    pub id: Snowflake,
    pub data: RecordData,
}

/// The RecordData is an enum containing all the options available for Records.
#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From)]
#[non_exhaustive]
pub enum RecordData {
    /// This record describes a media file, such as an image.
    Media(MediaRecord),
}

#[non_exhaustive]
pub enum RecordType {
    Media = 1,
}

#[non_exhaustive]
pub enum BooruId {
    Danbooru = 1,
}

impl RecordData {
    /// Get this record's type.
    /// Use this to quickly filter records from a stream.
    pub fn type_id(&self) -> RecordType {
        match self {
            RecordData::Media(_) => RecordType::Media,
        }
    }

    /// Get this record's entity type.
    ///
    /// This returns an integer with a meaning specific to the record type (see `type_id()`):
    /// - for Media records, it represents the state of the media (never downloaded, already downloaded, error)
    /// - for booru records, it represents what kind of booru entity it is (post, tag, comment)
    ///
    /// Use this with `type_id` to quickly filter records from a stream.
    pub fn entity_type(&self) -> u32 {
        match self {
            RecordData::Media(record) => record.get_state() as u32,
        }
    }

    /// Get this record's associated booru's ID.
    ///
    /// For booru record types, it returns a unique ID that identifies that booru.
    /// For record types across boorus (like Media) it returns the corresponding booru's ID.
    pub fn booru_id(&self) -> BooruId {
        use crate::record_types::media::resource_locator::MediaResourceLocator;
        match self {
            RecordData::Media(record) => match record.locator {
                MediaResourceLocator::Danbooru { .. } => BooruId::Danbooru,
            },
        }
    }
}
