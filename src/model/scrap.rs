use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize)]
pub struct ScrapedUser {
    #[serde(rename = "achievements")]
    pub medals: Box<[ScrapedMedal]>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScrapedMedal {
    #[serde(rename(serialize = "link"))]
    pub icon_url: Box<str>,
    #[serde(rename(serialize = "medalid"))]
    pub id: u16,
    pub name: Box<str>,
    pub grouping: Box<str>,
    pub ordering: u8,
    pub slug: Box<str>,
    pub description: Box<str>,
    #[serde(rename(serialize = "restriction"), serialize_with = "serialize_mode")]
    pub mode: Option<Box<str>>,
    pub instructions: Option<Box<str>>,
}

fn serialize_mode<S: Serializer>(value: &Option<Box<str>>, s: S) -> Result<S::Ok, S::Error> {
    let value = value.as_deref().unwrap_or("NULL");

    s.serialize_str(value)
}
