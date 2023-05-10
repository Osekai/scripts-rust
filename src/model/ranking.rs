use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Serialize, Serializer};
use time::OffsetDateTime;

use super::{MedalRarities, OsuUser};

#[derive(Serialize)]
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
    #[serde(rename(serialize = "rarest_medal"))]
    pub rarest_medal_id: u16,
    #[serde(serialize_with = "serialize_datetime")]
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
    #[serde(serialize_with = "bool_to_int")]
    pub restricted: bool,
}

impl RankingUser {
    pub fn new(user: OsuUser, rarities: &MedalRarities) -> Self {
        match user {
            OsuUser::Available(user) => {
                let stdev_acc = user.std_dev(|stats| stats.acc);
                let stdev_level = user.std_dev(|stats| stats.level);
                let stdev_pp = user.std_dev(|stats| stats.pp);

                let (rarest_medal_id, rarest_medal_achieved) = match user.rarest_medal(rarities) {
                    Some(medal) => (medal.medal_id as u16, medal.achieved_at),
                    None => (0, OffsetDateTime::from_unix_timestamp(0).unwrap()),
                };

                let [std, tko, ctb, mna] = user.inner;

                Self {
                    rarest_medal_id,
                    rarest_medal_achieved,
                    id: user.user_id,
                    name: user.username,
                    stdev_acc,
                    standard_acc: std.acc,
                    taiko_acc: tko.acc,
                    ctb_acc: ctb.acc,
                    mania_acc: mna.acc,
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

fn serialize_datetime<S: Serializer>(datetime: &OffsetDateTime, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(&DateTime(datetime))
}

fn bool_to_int<S: Serializer>(value: &bool, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u8(*value as u8)
}

struct DateTime<'a>(&'a OffsetDateTime);

impl Display for DateTime<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let date = self.0.date();
        let time = self.0.time();

        write!(
            f,
            "{}-{}-{} {}:{}:{}",
            date.year(),
            date.month() as u8,
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
        )
    }
}
