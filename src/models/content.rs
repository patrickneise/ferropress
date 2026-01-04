use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostMetadata {
    pub title: String,
    #[serde(with = "naivedate_ymd")]
    pub date: NaiveDate,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content: String,
    pub slug: String,
}

mod naivedate_ymd {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.format(FORMAT).to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(|e| {
            serde::de::Error::custom(format!("invalid date '{s}', expected YYYY-MM-DD: {e}"))
        })
    }
}
