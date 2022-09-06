use serde::Serialize;

#[derive(Serialize)]
pub struct MedalRarity {
    medal_id: u32,
    frequency: f64,
}
