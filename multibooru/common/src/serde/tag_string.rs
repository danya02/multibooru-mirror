use serde::{Deserializer, Serializer};

pub fn deserialize_tag_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
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

pub fn serialize_tag_string<S>(tags: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
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