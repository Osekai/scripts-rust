use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScrapedUser {
    #[serde(rename = "achievements")]
    pub medals: Box<[ScrapedMedal]>,
}

#[derive(Debug, Deserialize)]
pub struct ScrapedMedal {
    pub icon_url: Box<str>,
    pub id: u16,
    pub name: Box<str>,
    pub grouping: Box<str>,
    pub ordering: u8,
    pub description: Box<str>,
    pub mode: Option<Box<str>>,
    pub instructions: Option<Box<str>>,
}
