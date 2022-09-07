use std::collections::HashMap;

use serde::Serialize;

use super::{IntHasher, UserFull};

#[derive(Serialize)]
pub struct RankingUser {
    pub id: u32,
    pub name: String,
    pub total_pp: f32,
    pub stdev_pp: f32,
    pub standard_pp: f32,
    pub taiko_pp: f32,
    pub ctb_pp: f32,
    pub mania_pp: f32,
    pub medal_count: usize,
    pub rarest_medal_id: Option<u32>,
    pub country_code: String,
    pub standard_global: Option<u32>,
    pub taiko_global: Option<u32>,
    pub ctb_global: Option<u32>,
    pub mania_global: Option<u32>,
    pub badge_count: usize,
    pub ranked_maps: usize,
    pub loved_maps: usize,
    pub subscribers: usize,
    pub replays_watched: usize,
    pub avatar_url: String,
}

impl RankingUser {
    pub fn new(user: UserFull, rarities: &HashMap<u32, f64, IntHasher>) -> Self {
        let total_pp = user.total_pp();
        let stdev_pp = user.std_dev_pp();
        let rarest_medal_id = user.rarest_medal_id(&rarities);

        let [std, tko, ctb, mna] = user.inner;

        let std_stats = std.statistics.as_ref();
        let tko_stats = tko.statistics.as_ref();
        let ctb_stats = ctb.statistics.as_ref();
        let mna_stats = mna.statistics.as_ref();

        let replays_watched = std_stats.map_or(0, |stats| stats.replays_watched)
            + tko_stats.map_or(0, |stats| stats.replays_watched)
            + ctb_stats.map_or(0, |stats| stats.replays_watched)
            + mna_stats.map_or(0, |stats| stats.replays_watched);

        Self {
            id: std.user_id,
            name: std.username.into_string(),
            total_pp,
            stdev_pp,
            standard_pp: std_stats.map_or(0.0, |stats| stats.pp),
            taiko_pp: tko_stats.map_or(0.0, |stats| stats.pp),
            ctb_pp: ctb_stats.map_or(0.0, |stats| stats.pp),
            mania_pp: mna_stats.map_or(0.0, |stats| stats.pp),
            medal_count: std.medals.as_ref().map_or(0, Vec::len),
            rarest_medal_id,
            country_code: std.country_code.into_string(),
            standard_global: std_stats.and_then(|stats| stats.global_rank),
            taiko_global: tko_stats.and_then(|stats| stats.global_rank),
            ctb_global: ctb_stats.and_then(|stats| stats.global_rank),
            mania_global: mna_stats.and_then(|stats| stats.global_rank),
            badge_count: std.badges.as_ref().map_or(0, Vec::len),
            ranked_maps: std.ranked_mapset_count.map_or(0, |count| count as usize),
            loved_maps: std.loved_mapset_count.map_or(0, |count| count as usize),
            subscribers: std.follower_count.map_or(0, |count| count as usize),
            replays_watched: replays_watched as usize,
            avatar_url: std.avatar_url,
        }
    }
}
