use serde::{Serialize, Serializer};
use time::OffsetDateTime;

#[derive(Serialize)]
pub struct Badge {
    #[serde(serialize_with = "serialize_datetime")]
    pub awarded_at: OffsetDateTime,
    pub description: String,
    pub users: Vec<u32>,
    pub image_url: String,
    pub url: String,
}

pub fn serialize_datetime<S>(datetime: &OffsetDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    datetime.unix_timestamp_nanos().serialize(s)
}
