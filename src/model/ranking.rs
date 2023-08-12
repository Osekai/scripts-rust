use std::{num::NonZeroU32, vec::IntoIter};

use time::OffsetDateTime;

use super::{user::ModeStats, MedalRarities, OsuUser};

pub struct RankingUser {
    pub id: u32,
    pub name: Box<str>,
    pub ignore_acc: bool,
    pub medal_count: u16,
    pub rarest_medal_id: u16,
    pub rarest_medal_achieved: OffsetDateTime,
    pub country_code: Box<str>,
    pub badge_count: u16,
    pub ranked_maps: u16,
    pub loved_maps: u16,
    pub followers: u32,
    pub subscribers: u32,
    pub replays_watched: u32,
    pub avatar_url: Box<str>,
    pub kudosu: i32,
    pub restricted: bool,
    pub std: RankingMode,
    pub tko: RankingMode,
    pub ctb: RankingMode,
    pub mna: RankingMode,
}

impl RankingUser {
    pub fn new(user: OsuUser, rarities: &MedalRarities) -> Self {
        match user {
            OsuUser::Available(user) => {
                let (rarest_medal_id, rarest_medal_achieved) = match user.rarest_medal(rarities) {
                    Some(medal) => (medal.medal_id as u16, medal.achieved_at),
                    None => (0, OffsetDateTime::from_unix_timestamp(0).unwrap()),
                };

                let [std, tko, ctb, mna] = user.inner;

                let max_rank = std
                    .global_rank
                    .map_or(0, NonZeroU32::get)
                    .max(tko.global_rank.map_or(0, NonZeroU32::get))
                    .max(ctb.global_rank.map_or(0, NonZeroU32::get))
                    .max(mna.global_rank.map_or(0, NonZeroU32::get));

                let is_inactive = max_rank == 0;

                let max_playcount = std
                    .playcount
                    .max(tko.playcount)
                    .max(ctb.playcount)
                    .max(mna.playcount);

                const PLAYCOUNT_THRESHOLD: u32 = 500;

                Self {
                    rarest_medal_id,
                    rarest_medal_achieved,
                    id: user.user_id,
                    name: user.username,
                    ignore_acc: is_inactive || max_playcount < PLAYCOUNT_THRESHOLD,
                    medal_count: user.medals.len() as u16,
                    country_code: user.country_code,
                    badge_count: user.badges.len() as u16,
                    ranked_maps: user.maps_ranked,
                    loved_maps: user.maps_loved,
                    followers: user.followers,
                    subscribers: user.subscribers,
                    replays_watched: user.replays_watched,
                    avatar_url: user.avatar_url,
                    kudosu: user.kudosu,
                    restricted: false,
                    std: RankingMode::from(std),
                    tko: RankingMode::from(tko),
                    ctb: RankingMode::from(ctb),
                    mna: RankingMode::from(mna),
                }
            }
            OsuUser::Restricted { user_id } => Self {
                id: user_id,
                restricted: true,
                rarest_medal_achieved: OffsetDateTime::from_unix_timestamp(0).unwrap(),
                name: Default::default(),
                ignore_acc: Default::default(),
                medal_count: Default::default(),
                rarest_medal_id: Default::default(),
                country_code: Default::default(),
                badge_count: Default::default(),
                ranked_maps: Default::default(),
                loved_maps: Default::default(),
                followers: Default::default(),
                subscribers: Default::default(),
                replays_watched: Default::default(),
                kudosu: Default::default(),
                avatar_url: Default::default(),
                std: Default::default(),
                tko: Default::default(),
                ctb: Default::default(),
                mna: Default::default(),
            },
        }
    }

    pub fn std_dev_acc(&self) -> f32 {
        if self.ignore_acc {
            return 0.0;
        }

        let values = [self.std.acc, self.tko.acc, self.ctb.acc, self.mna.acc];

        std_dev(values)
    }

    pub fn std_dev_level(&self) -> f32 {
        let values = [
            self.std.level,
            self.tko.level,
            self.ctb.level,
            self.mna.level,
        ];

        std_dev(values)
    }

    pub fn std_dev_pp(&self) -> f32 {
        let values = [self.std.pp, self.tko.pp, self.ctb.pp, self.mna.pp];

        std_dev(values)
    }

    pub fn total_pp(&self) -> f32 {
        self.std.pp + self.tko.pp + self.ctb.pp + self.mna.pp
    }
}

fn std_dev(values: [f32; 4]) -> f32 {
    let total: f32 = values.iter().sum();
    let mean = total / 4.0;

    let variance: f32 = values
        .iter()
        .map(|value| (value - mean) * (value - mean))
        .sum();

    let std_dev = (variance / 3.0).sqrt();

    (total - 2.0 * std_dev).max(0.0)
}

#[derive(Default)]
pub struct RankingMode {
    pub acc: f32,
    pub global_rank: Option<NonZeroU32>,
    pub level: f32,
    pub pp: f32,
}

impl From<ModeStats> for RankingMode {
    fn from(stats: ModeStats) -> Self {
        Self {
            acc: stats.acc,
            global_rank: stats.global_rank,
            level: stats.level,
            pp: stats.pp,
        }
    }
}

pub struct RankingsIter {
    users: IntoIter<OsuUser>,
    rarities: MedalRarities,
}

impl RankingsIter {
    pub fn new(users: Vec<OsuUser>, rarities: MedalRarities) -> Self {
        Self {
            users: users.into_iter(),
            rarities,
        }
    }

    pub fn len(&self) -> usize {
        self.users.len()
    }
}

impl Iterator for RankingsIter {
    type Item = RankingUser;

    fn next(&mut self) -> Option<Self::Item> {
        self.users
            .next()
            .map(|user| RankingUser::new(user, &self.rarities))
    }
}
