use chrono::{DateTime, Utc};
use common::serde::tag_string::{deserialize_tag_string, serialize_tag_string};
use serde::{Serialize, Deserialize};

use crate::Rating;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct PostVersion {
    pub id: i64,
    pub post_id: i64,
    pub version: i32,

    pub updater_id: i64,
    pub updated_at: DateTime<Utc>,


    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string",
    )]
    pub tags: Vec<String>, // as string
    pub added_tags: Vec<String>,
    pub removed_tags: Vec<String>,
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string",
    )]
    pub unchanged_tags: Vec<String>, // as string

    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string",
    )]
    pub obsolete_added_tags: Vec<String>, // as string
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string",
    )]
    pub obsolete_removed_tags: Vec<String>, // as string

    pub rating: Rating,
    pub rating_changed: bool,

    pub source: String,
    pub source_changed: bool,

    pub parent_id: Option<i64>,
    pub parent_changed: bool,
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;

    use super::*;
    fn get_test_data() -> &'static str {
        r#"
        {"id":35923751,
        "post_id":5039418,
        "tags":"1boy basil_(omori) blonde_hair blue_eyes brown_vest camera closed_mouth collared_shirt english_commentary field flower flower_field from_side hair_flower hair_ornament highres holding holding_camera jeanty_art leaf omori orange_background orange_sky orange_theme outdoors pink_flower shirt short_sleeves sky solo sunflower sunset upper_body vest white_shirt yellow_flower",
        "added_tags":["orange_background","orange_theme"],
        "removed_tags":[],
        "updater_id":200729,
        "updated_at":"2022-01-08T13:41:49.000-05:00",
        "rating":"s",
        "rating_changed":false,
        "parent_id":null,
        "parent_changed":false,
        "source":"https://twitter.com/jeanty_art/status/1479025928562958338",
        "source_changed":false,
        "version":2,
        "obsolete_added_tags":"",
        "obsolete_removed_tags":"",
        "unchanged_tags":"1boy basil_(omori) blonde_hair blue_eyes brown_vest camera closed_mouth collared_shirt english_commentary field flower flower_field from_side hair_flower hair_ornament highres holding holding_camera jeanty_art leaf omori orange_sky outdoors pink_flower shirt short_sleeves sky solo sunflower sunset upper_body vest white_shirt yellow_flower"
    }"#
    }

    #[test]
    fn test_deserialize_danbooru_postversion() {
        let data = get_test_data();
        let common_tags = vec!["1boy","basil_(omori)","blonde_hair","blue_eyes","brown_vest","camera","closed_mouth","collared_shirt","english_commentary","field","flower","flower_field","from_side","hair_flower","hair_ornament","highres","holding","holding_camera","jeanty_art","leaf","omori","orange_sky","outdoors","pink_flower","shirt","short_sleeves","sky","solo","sunflower","sunset","upper_body","vest","white_shirt","yellow_flower"];
        let added_tags = vec!["orange_background","orange_theme"];
        let mut total_tags = common_tags.clone();
        total_tags.extend(added_tags.clone());
        total_tags.sort();
        let none: Vec<String> = vec![];

        let postversion: PostVersion = serde_json::from_str(data).unwrap();
        assert_eq!(postversion.id, 35923751);
        assert_eq!(postversion.post_id, 5039418);
        assert_eq!(postversion.version, 2);
        assert_eq!(postversion.updater_id, 200729);
        assert_eq!(postversion.updated_at, Utc.with_ymd_and_hms(2022, 1, 8, 13 + 5, 41, 49).unwrap());
        assert_eq!(postversion.tags, total_tags);
        assert_eq!(postversion.added_tags, added_tags);
        assert_eq!(postversion.removed_tags, none);
        assert_eq!(postversion.unchanged_tags, common_tags);
        assert_eq!(postversion.obsolete_added_tags, none);
        assert_eq!(postversion.obsolete_removed_tags, none);
        
        assert_eq!(postversion.rating, Rating::Sensitive);
        assert_eq!(postversion.rating_changed, false);
        assert_eq!(postversion.parent_id, None);
        assert_eq!(postversion.parent_changed, false);

        assert_eq!(postversion.source, "https://twitter.com/jeanty_art/status/1479025928562958338");
        assert_eq!(postversion.source_changed, false);
    }
}

