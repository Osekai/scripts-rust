use serde::Serialize;

#[derive(Serialize)]
pub struct Ranking {
    id: u32,
    name: String,
    total_pp: f64,
    stdev_pp: f64,
    standard_pp: f64,
    taiko_pp: f64,
    ctb_pp: f64,
    mania_pp: f64,
    medal_count: usize,
    rarest_medal_id: u32,
    country_code: String,
    standard_global: u32,
    taiko_global: u32,
    ctb_global: u32,
    mania_global: u32,
    badge_count: usize,
    ranked_maps: usize,
    loved_maps: usize,
    subscribers: usize,
    replays_watched: usize,
    avatar_url: String,
}
