use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::{
    record_types::{danbooru::DanbooruRecord, media::MediaRecord, rule34::Rule34Record},
    Snowflake,
};

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Record {
    pub id: Snowflake,
    pub data: RecordData,
}

impl Record {
    pub fn new(data: RecordData) -> Self {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let data_hash = hasher.finish();

        Self {
            data,
            id: Snowflake::new((data_hash % u16::MAX as u64) as u16),
        }
    }
}

/// The RecordData is an enum containing all the options available for Records.
#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From, Hash)]
#[non_exhaustive]
pub enum RecordData {
    /// This record describes a media file, such as an image.
    Media(MediaRecord),

    /// This record describes an entity that comes from Rule34.xxx.
    Rule34(Rule34Record),

    /// This record describes an entity that comes from Danbooru.donmai.us.
    Danbooru(DanbooruRecord),
}

#[non_exhaustive]
pub enum RecordType {
    Media = 1,
    Rule34 = 2,
    Danbooru = 3,
}

#[non_exhaustive]
pub enum BooruId {
    Danbooru = 1,
    Rule34 = 2,
}

impl RecordData {
    /// Get this record's type.
    /// Use this to quickly filter records from a stream.
    pub fn type_id(&self) -> RecordType {
        match self {
            RecordData::Media(_) => RecordType::Media,
            RecordData::Rule34(_) => RecordType::Rule34,
            RecordData::Danbooru(_) => RecordType::Danbooru,
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
            RecordData::Rule34(record) => record.entity_type() as u32,
            RecordData::Danbooru(record) => record.entity_type() as u32,
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
            RecordData::Rule34(_) => BooruId::Rule34,
            RecordData::Danbooru(_) => BooruId::Danbooru,
        }
    }

    /// Get this record's entity's ID.
    ///
    /// For booru record types, it is that entity's booru ID.
    /// For example, if it is a Danbooru post, then this returns the Danbooru post ID.
    ///
    /// For media, this returns a hash of the resource locator.
    pub fn entity_id(&self) -> u64 {
        match self {
            RecordData::Media(record) => record.locator.true_hash_as_u64(),
            RecordData::Rule34(record) => record.entity_id(),
            RecordData::Danbooru(record) => record.entity_id(),
        }
    }
}
