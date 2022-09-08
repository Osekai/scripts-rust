use serde::Serialize;

#[derive(Serialize)]
pub struct MedalRarity {
    pub medal_id: u32,
    pub total: usize,
    pub frequency: f64,
}
