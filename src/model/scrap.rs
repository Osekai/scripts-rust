use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize)]
pub struct ScrapedUser {
    #[serde(rename = "achievements")]
    pub medals: Vec<ScrapedMedal>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScrapedMedal {
    #[serde(rename(serialize = "link"))]
    pub icon_url: String,
    #[serde(rename(serialize = "medalid"))]
    pub id: u32,
    pub name: String,
    pub grouping: String,
    pub ordering: u8,
    pub slug: String,
    pub description: String,
    #[serde(rename(serialize = "restriction"), serialize_with = "serialize_mode")]
    pub mode: Option<String>,
    pub instructions: Option<String>,
}

fn serialize_mode<S: Serializer>(value: &Option<String>, s: S) -> Result<S::Ok, S::Error> {
    let value = value.as_deref().unwrap_or("NULL");

    s.serialize_str(value)
}
