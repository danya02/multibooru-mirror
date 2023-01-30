use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Rating {
    #[serde(rename = "g")]
    General,
    #[serde(rename = "s")]
    Sensitive,
    #[serde(rename = "q")]
    Questionable,
    #[serde(rename = "e")]
    Explicit,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Post {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>, // if this could be null, this means that no updates since initial,
    // so this should copy created_at
    pub uploader_id: u64, // maybe option
    pub approver_id: Option<u64>,

    #[serde(alias = "tag_string_general")]
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string"
    )]
    pub tags_general: Vec<String>,

    #[serde(alias = "tag_string_artist")]
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string"
    )]
    pub tags_artist: Vec<String>,

    #[serde(alias = "tag_string_copyright")]
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string"
    )]
    pub tags_copyright: Vec<String>,

    #[serde(alias = "tag_string_character")]
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string"
    )]
    pub tags_character: Vec<String>,

    #[serde(alias = "tag_string_meta")]
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string"
    )]
    pub tags_meta: Vec<String>,

    // The "tag_string" field should be the union of all the above fields.
    // This is supposed to be redundant, so we can use it to check that no tags are lost in the process.
    #[serde(
        deserialize_with = "deserialize_tag_string",
        serialize_with = "serialize_tag_string"
    )]
    #[serde(alias = "tag_string")]
    pub tags: Vec<String>,

    // The `tag_count*` fields are ignored, as they are expected to be redundant with the tag lists.
    pub rating: Rating,
    pub parent_id: Option<u64>,
    pub source: Option<String>,

    #[serde(deserialize_with = "deserialize_md5", serialize_with = "serialize_md5")]
    /// This can be missing. For example, it is missing on posts that have `is_banned=true`.
    /// It has been seen on posts that were automatically rejected, because they come from banned sources (`banned_artist`).
    pub md5: Option<[u8; 16]>,

    #[serde(alias = "file_url")]
    pub url: String, // corresponds to "file_url", previews are not recorded

    pub file_size: usize,
    pub file_ext: String,
    pub image_width: u32,
    pub image_height: u32,
    pub has_large: bool,

    pub score: i32,

    // these are not documented but appear in public responses
    pub up_score: Option<u32>,
    pub down_score: Option<u32>,

    pub fav_count: u32,

    pub last_commented_at: Option<DateTime<Utc>>,
    pub last_comment_bumped_at: Option<DateTime<Utc>>,
    pub last_noted_at: Option<DateTime<Utc>>,

    pub has_children: bool,
    // It is not clear what these two fields mean as compared to `has_children`.
    pub has_active_children: bool,
    pub has_visible_children: bool,

    pub pixiv_id: Option<u64>,

    pub is_pending: bool,
    pub is_flagged: bool,
    pub is_deleted: bool,
    pub is_banned: bool,

    pub bit_flags: u32,
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
                // A tag is a string of letters and numbers, and they are separated by whitespaces.
                .split(' ')
                // A string that is empty or contains only whitespaces is not a tag.
                .filter(|el| !el.is_empty())
                .filter(|el| !el.chars().all(|c| c.is_whitespace()))
                .map(|el| el.to_string())
                .collect::<Vec<String>>())
        }
    }
    deserializer.deserialize_str(JsonStringToVecVisitor)
}

fn serialize_tag_string<S>(tags: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tag_string = String::new();
    for tag in tags {
        tag_string.push_str(tag);
        // If there are more tags, separate them with a whitespace.
        if tag != tags.last().unwrap() {
            tag_string.push(' ');
        }
    }
    serializer.serialize_str(&tag_string)
}

fn deserialize_md5<'de, D>(deserializer: D) -> Result<Option<[u8; 16]>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, Visitor};
    use std::fmt;

    struct JsonStringToMd5ArrVisitor;
    impl<'de> Visitor<'de> for JsonStringToMd5ArrVisitor {
        type Value = Option<[u8; 16]>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("md5 hash as a hex string of length 32")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
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
                let byte_str = std::str::from_utf8(byte).or(Err(E::custom(format!(
                    "invalid utf8 encountered while slicing string into byte pieces: {byte:?}",
                ))))?;
                output[i] = u8::from_str_radix(byte_str, 16).or(Err(E::custom(format!(
                    "byte in position {i} is an invalid hexadecimal value: {byte_str}",
                ))))?;
            }
            Ok(Some(output))
        }
    }
    deserializer.deserialize_str(JsonStringToMd5ArrVisitor)
}

fn serialize_md5<S>(md5: &Option<[u8; 16]>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match md5 {
        Some(md5) => {
            let mut md5_str = String::new();
            for byte in md5 {
                md5_str.push_str(&format!("{byte:02x}"));
            }
            serializer.serialize_str(&md5_str)
        }
        None => serializer.serialize_none(),
    }
}

impl Post {
    /// Check that the `tag_string` field contains the union of the other tag fields.
    /// If this is not the case, then some tags are being missed --
    /// perhaps there are more tag types than we know about.
    pub fn tag_string_is_union(&self) -> bool {
        let mut tag_string_tags = self.tags.clone();
        tag_string_tags.sort();
        let mut union_tags = self.tags_general.clone();
        union_tags.extend(self.tags_artist.clone());
        union_tags.extend(self.tags_copyright.clone());
        union_tags.extend(self.tags_character.clone());
        union_tags.extend(self.tags_meta.clone());
        union_tags.sort();
        let result = tag_string_tags == union_tags;
        if !result {
            eprintln!(
                "!!! tag_string_is_union failed for post with id {} !!!",
                self.id
            );
            eprintln!("tag_string_tags: {tag_string_tags:?}");
            eprintln!("union_tags: {union_tags:?}");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

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

        let post: Post = serde_json::from_str(data)
            .expect("failed to deserialize JSON that should be valid -- error in test");
        assert_eq!(post.id, 4777148);
        assert_eq!(
            post.created_at,
            Utc.with_ymd_and_hms(2021, 9, 14, 14 + 4, 33, 47).unwrap()
                + Duration::milliseconds(088)
        ); // Their timezone is -4, so add 4h to get UTC
        assert_eq!(post.uploader_id, 350028);
        assert_eq!(post.score, 1);
        assert_eq!(
            post.source,
            Some("https://www.pokemon.jp/look/comic/detail/16524.html".to_string())
        );
        assert_eq!(
            post.md5,
            Some([
                0x6a, 0x12, 0xe4, 0x91, 0x32, 0x03, 0x16, 0x46, 0x92, 0x45, 0x96, 0xe2, 0x1f, 0x7c,
                0xca, 0xb9
            ])
        );
        assert_eq!(post.last_comment_bumped_at, None);
        assert_eq!(post.image_height, 1283);
        assert_eq!(post.image_width, 402);

        assert_eq!(post.rating, Rating::General);
        assert_eq!(post.image_width, 402);
        assert_eq!(post.image_height, 1283);
        assert_eq!(post.fav_count, 1);
        assert_eq!(
            post.last_noted_at,
            Some(
                Utc.with_ymd_and_hms(2021, 9, 24, 02 + 4, 45, 30).unwrap()
                    + Duration::milliseconds(453)
            )
        ); // Their timezone is -4, so add 4h to get UTC
        assert_eq!(post.parent_id, None);

        assert_eq!(post.has_children, false);
        assert_eq!(post.has_active_children, false);
        assert_eq!(post.has_visible_children, false);

        assert_eq!(post.approver_id, Some(728936));
        assert_eq!(post.file_size, 237334);
        assert_eq!(post.up_score, Some(1));
        assert_eq!(post.down_score, Some(0));

        assert_eq!(post.is_pending, false);
        assert_eq!(post.is_flagged, false);
        assert_eq!(post.is_deleted, false);
        assert_eq!(post.is_banned, false);

        assert_eq!(
            post.updated_at,
            Utc.with_ymd_and_hms(2022, 5, 23, 14 + 4, 12, 15).unwrap()
                + Duration::milliseconds(938)
        ); // Their timezone is -4, so add 4h to get UTC
        assert_eq!(post.pixiv_id, None);
        assert_eq!(post.last_commented_at, None);
        assert_eq!(
            post.tags_general,
            vec![
                "4koma",
                "blank_eyes",
                "blush",
                "bowl",
                "chopsticks",
                "comic",
                "company_name",
                "copyright_name",
                "food",
                "furigana",
                "mundane_utility",
                "nagashi_soumen",
                "narration",
                "no_humans",
                "noodles",
                "pokemon_(creature)",
                "soumen",
                "sound_effects",
                "sparkle",
                "speech_bubble",
                "sweatdrop",
                "water"
            ]
        );
        assert_eq!(
            post.tags_character,
            vec!["blastoise", "litten", "popplio", "rowlet"]
        );
        assert_eq!(post.tags_copyright, vec!["pokemon"]);
        assert_eq!(post.tags_artist, vec!["yamashita_takahiro"]);
        assert_eq!(
            post.tags_meta,
            vec!["highres", "official_art", "translated"]
        );
        assert!(post.tag_string_is_union());
        assert_eq!(
            post.url,
            "https://cdn.donmai.us/original/6a/12/6a12e49132031646924596e21f7ccab9.jpg"
        );
        assert_eq!(post.file_ext, "jpg");

        assert_eq!(post.has_large, false);

        assert_eq!(post.bit_flags, 2);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let data = get_test_json_string();
        // First deserialize the original JSON into a Post
        let post: Post = serde_json::from_str(data).expect("Error in deserializing Post from JSON");
        // Then serialize the Post into JSON
        let serialized = serde_json::to_string(&post).expect("Error in serializing Post to JSON");
        // Then deserialize the JSON back into a Post
        let post2: Post =
            serde_json::from_str(&serialized).expect("Error in deserializing Post from JSON");
        // The two posts should be equal
        assert_eq!(post, post2);
        // Finally, serialize the second Post into JSON
        let serialized2 = serde_json::to_string(&post2).expect("Error in serializing Post to JSON");
        // The two JSON strings should be equal
        assert_eq!(serialized, serialized2);
    }
}
