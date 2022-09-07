use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ScrapedUser {
    #[serde(rename = "achievements")]
    pub medals: Vec<ScrapedMedal>,
}

#[derive(Deserialize, Serialize)]
pub struct ScrapedMedal {
    #[serde(rename(serialize = "link"))]
    pub icon_url: String,
    #[serde(rename(serialize = "medalid"))]
    pub id: u32,
    pub name: String,
    pub grouping: String,
    pub ordering: u8,
    #[serde(skip_serializing)]
    pub slug: String,
    pub description: String,
    #[serde(rename(serialize = "restriction"))]
    pub mode: Option<String>,
    #[serde(skip_serializing)]
    pub instructions: Option<String>,
}
