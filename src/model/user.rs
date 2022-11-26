use rosu_v2::prelude::{Badge, CountryCode, MedalCompact, User, UserStatistics, Username};

use super::MedalRarities;

#[derive(Default)]
pub struct ModeStats {
    pub pp: f32,
    pub global_rank: Option<u32>,
}

impl From<Option<&UserStatistics>> for ModeStats {
    #[inline]
    fn from(stats: Option<&UserStatistics>) -> Self {
        match stats {
            Some(stats) => Self {
                pp: stats.pp,
                global_rank: stats.global_rank,
            },
            None => Self::default(),
        }
    }
}

pub struct UserFull {
    pub inner: [ModeStats; 4],
    pub avatar_url: Box<str>,
    pub badges: Box<[Badge]>,
    pub country_code: CountryCode,
    pub followers: u32,
    pub maps_ranked: u16,
    pub maps_loved: u16,
    pub medals: Box<[MedalCompact]>,
    pub replays_watched: u32,
    pub subscribers: u32,
    pub user_id: u32,
    pub username: Username,
}

impl UserFull {
    pub fn new(std: User, tko: User, ctb: User, mna: User) -> Self {
        let avatar_url = std.avatar_url.into_boxed_str();
        let badges = std.badges.unwrap_or_default().into_boxed_slice();
        let country_code = std.country_code;
        let followers = std.follower_count.unwrap_or(0);
        let maps_ranked = std.ranked_mapset_count.map_or(0, |count| count as u16);
        let maps_loved = std.loved_mapset_count.map_or(0, |count| count as u16);
        let medals = std.medals.unwrap_or_default().into_boxed_slice();
        let subscribers = std.mapping_follower_count.unwrap_or(0);
        let user_id = std.user_id;
        let username = std.username;

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
            avatar_url,
            badges,
            country_code,
            followers,
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

    /// Iterate over the pp values for each mode
    pub fn pp_iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.inner.iter().map(|stats| stats.pp)
    }

    /// Sum up the pp values of each mode
    pub fn total_pp(&self) -> f32 {
        self.pp_iter().sum()
    }

    /// Calculate the standard deviation for the four pp values
    pub fn std_dev_pp(&self) -> f32 {
        let total = self.total_pp();
        let mean = total / 4.0;
        let variance: f32 = self.pp_iter().map(|pp| (pp - mean) * (pp - mean)).sum();
        let std_dev = (variance / 3.0).sqrt();

        total - 2.0 * std_dev
    }
}
