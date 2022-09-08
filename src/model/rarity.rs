use serde::Serialize;

#[derive(Serialize)]
pub struct MedalRarity {
    #[serde(rename(serialize = "medalid"))]
    pub medal_id: u32,
    pub count: usize,
    pub frequency: f64,
}
