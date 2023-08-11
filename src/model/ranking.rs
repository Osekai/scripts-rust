use time::OffsetDateTime;

use super::{MedalRarities, OsuUser};

pub struct RankingUser {
    pub id: u32,
    pub name: Box<str>,
    pub stdev_acc: f32,
    pub standard_acc: f32,
    pub taiko_acc: f32,
    pub ctb_acc: f32,
    pub mania_acc: f32,
    pub stdev_level: f32,
    pub standard_level: f32,
    pub taiko_level: f32,
    pub ctb_level: f32,
    pub mania_level: f32,
    pub stdev_pp: f32,
    pub standard_pp: f32,
    pub taiko_pp: f32,
    pub ctb_pp: f32,
    pub mania_pp: f32,
    pub medal_count: u16,
    pub rarest_medal_id: u16,
    pub rarest_medal_achieved: OffsetDateTime,
    pub country_code: Box<str>,
    pub standard_global: Option<u32>,
    pub taiko_global: Option<u32>,
    pub ctb_global: Option<u32>,
    pub mania_global: Option<u32>,
    pub badge_count: u16,
    pub ranked_maps: u16,
    pub loved_maps: u16,
    pub followers: u32,
    pub subscribers: u32,
    pub replays_watched: u32,
    pub avatar_url: Box<str>,
    pub kudosu: i32,
    pub restricted: bool,
}

impl RankingUser {
    pub fn new(user: OsuUser, rarities: &MedalRarities) -> Self {
        match user {
            OsuUser::Available(user) => {
                let mut stdev_acc = user.std_dev(|stats| stats.acc);
                let stdev_level = user.std_dev(|stats| stats.level);
                let stdev_pp = user.std_dev(|stats| stats.pp);

                let (rarest_medal_id, rarest_medal_achieved) = match user.rarest_medal(rarities) {
                    Some(medal) => (medal.medal_id as u16, medal.achieved_at),
                    None => (0, OffsetDateTime::from_unix_timestamp(0).unwrap()),
                };

                let [std, tko, ctb, mna] = user.inner;

                let mut standard_acc = std.acc;
                let mut taiko_acc = tko.acc;
                let mut ctb_acc = ctb.acc;
                let mut mania_acc = mna.acc;

                let max_rank = std
                    .global_rank
                    .unwrap_or(0)
                    .max(tko.global_rank.unwrap_or(0))
                    .max(ctb.global_rank.unwrap_or(0))
                    .max(mna.global_rank.unwrap_or(0));

                let is_inactive = max_rank == 0;

                let max_playcount = std
                    .playcount
                    .max(tko.playcount)
                    .max(ctb.playcount)
                    .max(mna.playcount);

                const PLAYCOUNT_THRESHOLD: u32 = 500;

                if is_inactive || max_playcount < PLAYCOUNT_THRESHOLD {
                    stdev_acc = 0.0;
                    standard_acc = 0.0;
                    taiko_acc = 0.0;
                    ctb_acc = 0.0;
                    mania_acc = 0.0;
                }

                Self {
                    rarest_medal_id,
                    rarest_medal_achieved,
                    id: user.user_id,
                    name: user.username,
                    stdev_acc,
                    standard_acc,
                    taiko_acc,
                    ctb_acc,
                    mania_acc,
                    stdev_level,
                    standard_level: std.level,
                    taiko_level: tko.level,
                    ctb_level: ctb.level,
                    mania_level: mna.level,
                    stdev_pp,
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
                    followers: user.followers,
                    subscribers: user.subscribers,
                    replays_watched: user.replays_watched,
                    avatar_url: user.avatar_url,
                    kudosu: user.kudosu,
                    restricted: false,
                }
            }
            OsuUser::Restricted { user_id } => Self {
                id: user_id,
                restricted: true,
                rarest_medal_achieved: OffsetDateTime::from_unix_timestamp(0).unwrap(),
                name: Default::default(),
                stdev_acc: Default::default(),
                standard_acc: Default::default(),
                taiko_acc: Default::default(),
                ctb_acc: Default::default(),
                mania_acc: Default::default(),
                stdev_level: Default::default(),
                standard_level: Default::default(),
                taiko_level: Default::default(),
                ctb_level: Default::default(),
                mania_level: Default::default(),
                stdev_pp: Default::default(),
                standard_pp: Default::default(),
                taiko_pp: Default::default(),
                ctb_pp: Default::default(),
                mania_pp: Default::default(),
                medal_count: Default::default(),
                rarest_medal_id: Default::default(),
                country_code: Default::default(),
                standard_global: Default::default(),
                taiko_global: Default::default(),
                ctb_global: Default::default(),
                mania_global: Default::default(),
                badge_count: Default::default(),
                ranked_maps: Default::default(),
                loved_maps: Default::default(),
                followers: Default::default(),
                subscribers: Default::default(),
                replays_watched: Default::default(),
                kudosu: Default::default(),
                avatar_url: Default::default(),
            },
        }
    }
}
