use std::num::NonZeroU32;

use rosu_v2::prelude::{Badge, MedalCompact, UserExtended, UserStatistics};

use super::MedalRarities;

pub enum OsuUser {
    Available(UserFull),
    Restricted { user_id: u32 },
}

#[derive(Default)]
pub struct ModeStats {
    pub acc: f32,
    pub level: f32,
    pub global_rank: Option<NonZeroU32>,
    pub playcount: u32,
    pub pp: f32,
}

impl From<Option<&UserStatistics>> for ModeStats {
    #[inline]
    fn from(stats: Option<&UserStatistics>) -> Self {
        match stats {
            Some(stats) => Self {
                acc: stats.accuracy,
                level: stats.level.float(),
                global_rank: stats.global_rank.and_then(NonZeroU32::new),
                playcount: stats.playcount,
                pp: stats.pp,
            },
            None => Self::default(),
        }
    }
}

pub struct UserFull {
    pub inner: [ModeStats; 4],
    pub badges: Box<[Badge]>,
    pub country_code: Box<str>,
    pub maps_ranked: u16,
    pub maps_loved: u16,
    pub medals: Box<[MedalCompact]>,
    pub replays_watched: u32,
    pub subscribers: u32,
    pub user_id: u32,
    pub username: Box<str>,
}

impl UserFull {
    pub fn new(std: UserExtended, tko: UserExtended, ctb: UserExtended, mna: UserExtended) -> Self {
        let badges = std.badges.unwrap_or_default().into_boxed_slice();
        let country_code = std.country_code.into_string().into_boxed_str();
        let maps_ranked = std.ranked_mapset_count.map_or(0, |count| count as u16);
        let maps_loved = std.loved_mapset_count.map_or(0, |count| count as u16);
        let medals = std.medals.unwrap_or_default().into_boxed_slice();
        let subscribers = std.mapping_follower_count.unwrap_or(0);
        let user_id = std.user_id;
        let username = std.username.into_string().into_boxed_str();

        let std = std.statistics.as_ref();
        let tko = tko.statistics.as_ref();
        let ctb = ctb.statistics.as_ref();
        let mna = mna.statistics.as_ref();

        let replays_watched = std.map_or(0, |stats| stats.replays_watched)
            + tko.map_or(0, |stats| stats.replays_watched)
            + ctb.map_or(0, |stats| stats.replays_watched)
            + mna.map_or(0, |stats| stats.replays_watched);

        Self {
            inner: [std.into(), tko.into(), ctb.into(), mna.into()],
            badges,
            country_code,
            maps_ranked,
            maps_loved,
            medals,
            replays_watched,
            subscribers,
            user_id,
            username,
        }
    }

    pub fn rarest_medal<'s>(&'s self, rarities: &MedalRarities) -> Option<&'s MedalCompact> {
        self.medals
            .iter()
            .flat_map(|medal| Some((medal, rarities.get(&(medal.medal_id as u16))?.count)))
            .min_by_key(|(_, count)| *count)
            .map(|(medal, _)| medal)
    }
}
