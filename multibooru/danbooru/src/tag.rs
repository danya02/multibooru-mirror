use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// A tag on Danbooru is identified by a numeric ID,
/// but that is seldom used.
/// In particular, posts are identified by their tags' names.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub id: u64,
    pub name: String,
    pub post_count: u64,
    pub category: TagCategory,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>, // If never updated, this is the same as created_at
    pub is_deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum TagCategory {
    General = 0,
    Artist = 1,
    // Number 2 appears to be unused as per https://danbooru.donmai.us/wiki_pages/api%3Atags
    Copyright = 3,
    Character = 4,
    Meta = 5,
}

#[cfg(test)]
mod test {
    use chrono::{Duration, TimeZone};

    use super::*;

    #[test]
    fn test_deserialize_danbooru_tag_json() {
        let data = r#"
        {
            "id": 2716,
            "name": "cat",
            "post_count": 53522,
            "category": 0,
            "created_at": "2013-02-28T00:28:48.044-05:00",
            "updated_at": "2019-08-30T17:23:14.479-04:00",
            "is_deprecated": false,
            "words": [
              "cat"
            ]
          }
        "#;

        let tag: Tag = serde_json::from_str(data).unwrap();
        assert_eq!(tag.id, 2716);
        assert_eq!(tag.name, "cat");
        assert_eq!(tag.post_count, 53522);
        assert_eq!(tag.category, TagCategory::General);
        assert_eq!(
            tag.created_at,
            Utc.with_ymd_and_hms(2013, 02, 28, 00 + 5, 28, 48).unwrap()
                + Duration::milliseconds(044)
        );
        assert_eq!(
            tag.updated_at,
            Utc.with_ymd_and_hms(2019, 08, 30, 17 + 4, 23, 14).unwrap()
                + Duration::milliseconds(479)
        );
        assert_eq!(tag.is_deprecated, false);
    }
}
