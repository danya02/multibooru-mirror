use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Rating {
    #[serde(alias = "g")]
    General,
    #[serde(alias = "s")]
    Sensitive,
    #[serde(alias = "q")]
    Questionable,
    #[serde(alias = "e")]
    Explicit,
}

#[derive(Serialize, Deserialize)]
pub struct Post {
    id: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>, // if this could be null, this means that no updates since initial,
    // so this should copy created_at
    uploader_id: u64, // maybe option
    approver_id: Option<u64>,

    #[serde(alias = "tag_string_general")]
    #[serde(deserialize_with = "deserialize_tag_string")]
    tags_general: Vec<String>,

    #[serde(alias = "tag_string_artist")]
    #[serde(deserialize_with = "deserialize_tag_string")]
    tags_artist: Vec<String>,

    #[serde(alias = "tag_string_copyright")]
    #[serde(deserialize_with = "deserialize_tag_string")]
    tags_copyright: Vec<String>,

    #[serde(alias = "tag_string_character")]
    #[serde(deserialize_with = "deserialize_tag_string")]
    tags_character: Vec<String>,

    #[serde(alias = "tag_string_meta")]
    #[serde(deserialize_with = "deserialize_tag_string")]
    tags_meta: Vec<String>,
    // could there be tags other than these?
    rating: Rating,
    parent_id: Option<u64>,
    source: Option<String>,

    #[serde(deserialize_with = "deserialize_md5")]
    md5: [u8; 16],

    #[serde(alias = "file_url")]
    url: String, // corresponds to "file_url", previews are not recorded

    file_size: usize,
    image_width: u32,
    image_height: u32,

    score: i32,

    // these are not documented but appear in public responses
    up_score: Option<u32>,
    down_score: Option<u32>,

    fav_count: u32,

    last_commented_at: Option<DateTime<Utc>>,
    last_noted_at: Option<DateTime<Utc>>,

    has_children: bool,
}

fn deserialize_tag_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, Visitor};
    use std::fmt;

    struct JsonStringToVecVisitor;
    impl<'de> Visitor<'de> for JsonStringToVecVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string with tags separated by whitespaces")
        }

        fn visit_str<E>(self, tags: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(tags
                .split(" ")
                .map(|el| el.to_string())
                .collect::<Vec<String>>())
        }
    }
    deserializer.deserialize_str(JsonStringToVecVisitor)
}

fn deserialize_md5<'de, D>(deserializer: D) -> Result<[u8; 16], D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, Visitor};
    use std::fmt;

    struct JsonStringToMd5ArrVisitor;
    impl<'de> Visitor<'de> for JsonStringToMd5ArrVisitor {
        type Value = [u8; 16];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("md5 hash as a string of length 32")
        }

        fn visit_str<E>(self, md5_str: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let bytes = md5_str.as_bytes();
            let mut output = [0; 16];
            if bytes.len() == 32 {
                for i in (0..bytes.len()).step_by(2) {
                    output[i / 2] = bytes[i] + bytes[i + 1];
                }
            } else {
                return Err(E::custom(format!(
                    "md5 hash length ({}) does not equal 32",
                    bytes.len()
                )));
            }
            Ok(output)
        }
    }
    deserializer.deserialize_str(JsonStringToMd5ArrVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// As a separate function to avoid clutter
    fn get_test_json_string() -> &'static str {
        r#"{
        "id":4777148,
        "created_at":"2021-09-14T14:33:47.088-04:00",
        "uploader_id":350028,
        "score":1,
        "source":"https://www.pokemon.jp/look/comic/detail/16524.html",
        "md5":"6a12e49132031646924596e21f7ccab9",
        "last_comment_bumped_at":null,
        "rating":"g",
        "image_width":402,
        "image_height":1283,
        "tag_string":"4koma blank_eyes blastoise blush bowl chopsticks comic company_name copyright_name food furigana highres litten mundane_utility nagashi_soumen narration no_humans noodles official_art pokemon pokemon_(creature) popplio rowlet soumen sound_effects sparkle speech_bubble sweatdrop translated water yamashita_takahiro",
        "fav_count":1,
        "file_ext":"jpg",
        "last_noted_at":"2021-09-24T02:45:30.453-04:00",
        "parent_id":null,
        "has_children":false,
        "approver_id":728936,
        "tag_count_general":22,
        "tag_count_artist":1,
        "tag_count_character":4,
        "tag_count_copyright":1,
        "file_size":237334,
        "up_score":1,
        "down_score":0,
        "is_pending":false,
        "is_flagged":false,
        "is_deleted":false,
        "tag_count":31,
        "updated_at":"2022-05-23T14:12:15.938-04:00",
        "is_banned":false,
        "pixiv_id":null,
        "last_commented_at":null,
        "has_active_children":false,
        "bit_flags":2,
        "tag_count_meta":3,
        "has_large":false,
        "has_visible_children":false,
        "tag_string_general":"4koma blank_eyes blush bowl chopsticks comic company_name copyright_name food furigana mundane_utility nagashi_soumen narration no_humans noodles pokemon_(creature) soumen sound_effects sparkle speech_bubble sweatdrop water",
        "tag_string_character":"blastoise litten popplio rowlet",
        "tag_string_copyright":"pokemon",
        "tag_string_artist":"yamashita_takahiro",
        "tag_string_meta":"highres official_art translated",
        "file_url":"https://cdn.donmai.us/original/6a/12/6a12e49132031646924596e21f7ccab9.jpg",
        "large_file_url":"https://cdn.donmai.us/original/6a/12/6a12e49132031646924596e21f7ccab9.jpg",
        "preview_file_url":"https://cdn.donmai.us/preview/6a/12/6a12e49132031646924596e21f7ccab9.jpg"
        }"#
    }

    #[test]
    fn test_deserialize_danbooru_post_json() {
        let data = get_test_json_string();
        let _post: Post = serde_json::from_str(data).expect("failed to deserialize JSON that should be valid -- error in test");
    }

}
