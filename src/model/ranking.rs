use rosu_v2::prelude::{CountryCode, Username};
use serde::Serialize;

use super::{MedalRarities, UserFull};

#[derive(Serialize)]
pub struct RankingUser {
    pub id: u32,
    pub name: Username,
    pub total_pp: f32,
    pub stdev_pp: f32,
    pub standard_pp: f32,
    pub taiko_pp: f32,
    pub ctb_pp: f32,
    pub mania_pp: f32,
    pub medal_count: u16,
    #[serde(rename(serialize = "rarest_medal"))]
    pub rarest_medal_id: u16,
    pub country_code: CountryCode,
    pub standard_global: Option<u32>,
    pub taiko_global: Option<u32>,
    pub ctb_global: Option<u32>,
    pub mania_global: Option<u32>,
    pub badge_count: u16,
    pub ranked_maps: u16,
    pub loved_maps: u16,
    pub subscribers: u32,
    pub replays_watched: u32,
    pub avatar_url: Box<str>,
}

impl RankingUser {
    pub fn new(user: UserFull, rarities: &MedalRarities) -> Self {
        let total_pp = user.total_pp();
        let stdev_pp = user.std_dev_pp();
        let rarest_medal_id = user.rarest_medal_id(rarities).map_or(0, |id| id as u16);

        let [std, tko, ctb, mna] = user.inner;

        Self {
            total_pp,
            stdev_pp,
            rarest_medal_id,
            id: user.user_id,
            name: user.username,
            standard_pp: std.pp,
            taiko_pp: tko.pp,
            ctb_pp: ctb.pp,
            mania_pp: mna.pp,
            medal_count: user.medals.len() as u16,
            country_code: user.country_code,
            standard_global: std.global_rank,
            taiko_global: tko.global_rank,
            ctb_global: ctb.global_rank,
            mania_global: mna.global_rank,
            badge_count: user.badges.len() as u16,
            ranked_maps: user.maps_ranked,
            loved_maps: user.maps_loved,
            subscribers: user.followers,
            replays_watched: user.replays_watched,
            avatar_url: user.avatar_url,
        }
    }
}
