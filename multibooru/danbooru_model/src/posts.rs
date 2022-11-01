use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

    pixiv_id: Option<u64>,
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
            let mut output = [0; 16];
            if md5_str.len() != 32 {
                return Err(E::custom(format!(
                    "md5 hash length ({}) does not equal 32",
                    md5_str.len()
                )));
            }
            for (i, byte) in md5_str.as_bytes().chunks(2).enumerate() {
                let byte_str = std::str::from_utf8(byte).or(Err(E::custom(format!("invalid utf8 encountered while slicing string into byte pieces: {:?}", byte))))?;
                output[i] = u8::from_str_radix(byte_str, 16).or(Err(E::custom(format!("byte in position {} is an invalid hexadecimal value: {}", i, byte_str))))?;
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
        let post: Post = serde_json::from_str(data).expect("failed to deserialize JSON that should be valid -- error in test");
        assert_eq!(post.id, 4777148);
        assert_eq!(post.created_at, Utc.ymd(2021, 9, 14).and_hms_milli(18, 33, 47, 088));
        assert_eq!(post.uploader_id, 350028);
        assert_eq!(post.score, 1);
        assert_eq!(post.source, Some("https://www.pokemon.jp/look/comic/detail/16524.html".to_string()));
        assert_eq!(post.md5, [0x6a, 0x12, 0xe4, 0x91, 0x32, 0x03, 0x16, 0x46, 0x92, 0x45, 0x96, 0xe2, 0x1f, 0x7c, 0xca, 0xb9]);

        assert_eq!(post.rating, Rating::General);
        assert_eq!(post.image_width, 402);
        assert_eq!(post.image_height, 1283);
        assert_eq!(post.fav_count, 1);
        assert_eq!(post.last_noted_at, Some(Utc.ymd(2021, 9, 24).and_hms_milli(06, 45, 30, 453)));
        assert_eq!(post.parent_id, None);
        assert_eq!(post.has_children, false);
        assert_eq!(post.approver_id, Some(728936));
        assert_eq!(post.file_size, 237334);
        assert_eq!(post.up_score, Some(1));
        assert_eq!(post.down_score, Some(0));
        /*
        assert_eq!(post.is_pending, false);
        assert_eq!(post.is_flagged, false);
        assert_eq!(post.is_deleted, false);
        assert_eq!(post.is_banned, false);
        */
        assert_eq!(post.updated_at, Utc.ymd(2022, 5, 23).and_hms_milli(18, 12, 15, 938));
        assert_eq!(post.pixiv_id, None);
        assert_eq!(post.last_commented_at, None);
        assert_eq!(post.tags_general, vec!["4koma", "blank_eyes", "blush", "bowl", "chopsticks", "comic", "company_name", "copyright_name", "food", "furigana", "mundane_utility", "nagashi_soumen", "narration", "no_humans", "noodles", "pokemon_(creature)", "soumen", "sound_effects", "sparkle", "speech_bubble", "sweatdrop", "water"]);
        assert_eq!(post.tags_character, vec!["blastoise", "litten", "popplio", "rowlet"]);
        assert_eq!(post.tags_copyright, vec!["pokemon"]);
        assert_eq!(post.tags_artist, vec!["yamashita_takahiro"]);
        assert_eq!(post.tags_meta, vec!["highres", "official_art", "translated"]);
        assert_eq!(post.url, "https://cdn.donmai.us/original/6a/12/6a12e49132031646924596e21f7ccab9.jpg");


    }

}
