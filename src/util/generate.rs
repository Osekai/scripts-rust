#![cfg(debug_assertions)]
#![allow(unused)]

use std::ops::Range;

use rand::{distributions::Alphanumeric, Rng};
use rosu_v2::prelude::{
    AccountHistory, Badge, CountryCode, GameMode, GradeCounts, Group, HistoryType, MedalCompact,
    MonthlyCount, Playstyle, ProfilePage, User, UserCover, UserKudosu, UserLevel, UserPage,
    UserStatistics, Username,
};
use time::{Date, OffsetDateTime};

use crate::model::UserFull;

pub trait Generate {
    fn generate<R: Rng>(rng: &mut R) -> Self;
}

pub trait GenerateRange {
    fn generate_range<R: Rng>(rng: &mut R, range: Range<usize>) -> Self;
}

impl Generate for bool {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        rng.gen_bool(0.5)
    }
}

macro_rules! impl_generate {
    ($($ty:ty),*) => {
        $(
            impl Generate for $ty {
                fn generate<R: Rng>(rng: &mut R) -> Self {
                    rng.gen()
                }
            }
        )*
    };
}

impl_generate!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

impl<T: Generate> Generate for Option<T> {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        if rng.gen_bool(0.5) {
            Some(Generate::generate(rng))
        } else {
            None
        }
    }
}

impl<T: Generate> GenerateRange for Vec<T> {
    fn generate_range<R: Rng>(rng: &mut R, range: Range<usize>) -> Self {
        let len = rng.gen_range(range);

        (0..len).map(|_| Generate::generate(rng)).collect()
    }
}

impl<T: Generate> GenerateRange for Option<Vec<T>> {
    fn generate_range<R: Rng>(rng: &mut R, range: Range<usize>) -> Self {
        rng.gen_bool(0.7)
            .then(|| GenerateRange::generate_range(rng, range))
    }
}

impl GenerateRange for String {
    fn generate_range<R: Rng>(rng: &mut R, range: Range<usize>) -> Self {
        let len = rng.gen_range(range);

        (0..len)
            .map(|_| rng.sample(Alphanumeric))
            .map(char::from)
            .collect()
    }
}

impl GenerateRange for Option<String> {
    fn generate_range<R: Rng>(rng: &mut R, range: Range<usize>) -> Self {
        rng.gen_bool(0.7)
            .then(|| GenerateRange::generate_range(rng, range))
    }
}

impl Generate for UserCover {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            custom_url: None,
            url: GenerateRange::generate_range(rng, 10..20),
            id: None,
        }
    }
}

impl Generate for OffsetDateTime {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self::from_unix_timestamp(rng.gen_range(1_000_000..100_000_000)).unwrap()
    }
}

impl Generate for UserKudosu {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            available: Generate::generate(rng),
            total: Generate::generate(rng),
        }
    }
}

impl Generate for GameMode {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        match rng.gen_range(0_u8..4) {
            0 => Self::Osu,
            1 => Self::Taiko,
            2 => Self::Catch,
            3 => Self::Mania,
            _ => unreachable!(),
        }
    }
}

impl Generate for Playstyle {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        match rng.gen_range(0..4) {
            0 => Self::Mouse,
            1 => Self::Keyboard,
            2 => Self::Tablet,
            3 => Self::Touch,
            _ => unreachable!(),
        }
    }
}

impl Generate for ProfilePage {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        match rng.gen_range(0..7) {
            0 => Self::Beatmaps,
            1 => Self::Historical,
            2 => Self::Kudosu,
            3 => Self::Me,
            4 => Self::Medals,
            5 => Self::RecentActivity,
            6 => Self::TopRanks,
            _ => unreachable!(),
        }
    }
}

impl Generate for AccountHistory {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            id: Generate::generate(rng),
            history_type: Generate::generate(rng),
            timestamp: Generate::generate(rng),
            seconds: Generate::generate(rng),
        }
    }
}

impl Generate for HistoryType {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        match rng.gen_range(0..3) {
            0 => Self::Note,
            1 => Self::Restriction,
            2 => Self::Silence,
            _ => unreachable!(),
        }
    }
}

impl Generate for Group {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            color: GenerateRange::generate_range(rng, 3..10),
            description: GenerateRange::generate_range(rng, 5..20),
            has_modes: Generate::generate(rng),
            id: Generate::generate(rng),
            identifier: GenerateRange::generate_range(rng, 16..17),
            is_probationary: Generate::generate(rng),
            modes: GenerateRange::generate_range(rng, 0..4),
            name: GenerateRange::generate_range(rng, 3..16),
            short_name: GenerateRange::generate_range(rng, 3..8),
        }
    }
}

impl Generate for Date {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        let year = rng.gen_range(2012..2023);
        let ordinal = rng.gen_range(1..366);

        Self::from_ordinal_date(year, ordinal).unwrap()
    }
}

impl Generate for MonthlyCount {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            start_date: Generate::generate(rng),
            count: Generate::generate(rng),
        }
    }
}

impl Generate for UserPage {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            html: GenerateRange::generate_range(rng, 0..500),
            raw: GenerateRange::generate_range(rng, 0..600),
        }
    }
}

impl Generate for GradeCounts {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            ss: Generate::generate(rng),
            ssh: Generate::generate(rng),
            s: Generate::generate(rng),
            sh: Generate::generate(rng),
            a: Generate::generate(rng),
        }
    }
}

impl Generate for UserLevel {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            current: Generate::generate(rng),
            progress: Generate::generate(rng),
        }
    }
}

impl Generate for UserStatistics {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            accuracy: Generate::generate(rng),
            country_rank: Generate::generate(rng),
            global_rank: Generate::generate(rng),
            grade_counts: Generate::generate(rng),
            is_ranked: Generate::generate(rng),
            level: Generate::generate(rng),
            max_combo: Generate::generate(rng),
            playcount: Generate::generate(rng),
            playtime: Generate::generate(rng),
            pp: Generate::generate(rng),
            ranked_score: Generate::generate(rng),
            replays_watched: Generate::generate(rng),
            total_hits: Generate::generate(rng),
            total_score: Generate::generate(rng),
        }
    }
}

impl Generate for Badge {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            awarded_at: Generate::generate(rng),
            description: GenerateRange::generate_range(rng, 5..20),
            image_url: GenerateRange::generate_range(rng, 15..20),
            url: GenerateRange::generate_range(rng, 15..20),
        }
    }
}

impl Generate for MedalCompact {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            achieved_at: Generate::generate(rng),
            medal_id: rng.gen_range(1..300),
        }
    }
}

impl Generate for CountryCode {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        (0..2)
            .map(|_| rng.gen_range(b'A'..b'Z'))
            .map(char::from)
            .collect()
    }
}

impl Generate for Username {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        let len = rng.gen_range(3..16);

        (0..len)
            .map(|_| rng.sample(Alphanumeric))
            .map(char::from)
            .collect()
    }
}

impl Generate for User {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        Self {
            avatar_url: GenerateRange::generate_range(rng, 15..25),
            comments_count: Generate::generate(rng),
            country: GenerateRange::generate_range(rng, 4..15),
            country_code: Generate::generate(rng),
            cover: Generate::generate(rng),
            default_group: GenerateRange::generate_range(rng, 5..25),
            discord: GenerateRange::generate_range(rng, 3..12),
            has_supported: Generate::generate(rng),
            interests: GenerateRange::generate_range(rng, 3..25),
            is_active: Generate::generate(rng),
            is_bot: Generate::generate(rng),
            is_deleted: Generate::generate(rng),
            is_online: Generate::generate(rng),
            is_supporter: Generate::generate(rng),
            join_date: Generate::generate(rng),
            kudosu: Generate::generate(rng),
            last_visit: Generate::generate(rng),
            location: GenerateRange::generate_range(rng, 3..10),
            max_blocks: Generate::generate(rng),
            max_friends: Generate::generate(rng),
            mode: Generate::generate(rng),
            occupation: GenerateRange::generate_range(rng, 3..20),
            playstyle: GenerateRange::generate_range(rng, 0..2),
            pm_friends_only: Generate::generate(rng),
            forum_post_count: Generate::generate(rng),
            profile_color: GenerateRange::generate_range(rng, 3..10),
            profile_order: GenerateRange::generate_range(rng, 7..8),
            title: GenerateRange::generate_range(rng, 4..15),
            title_url: GenerateRange::generate_range(rng, 15..20),
            twitter: GenerateRange::generate_range(rng, 3..15),
            user_id: Generate::generate(rng),
            username: Generate::generate(rng),
            website: GenerateRange::generate_range(rng, 10..20),
            account_history: GenerateRange::generate_range(rng, 0..25),
            badges: Some(GenerateRange::generate_range(rng, 0..20)),
            beatmap_playcounts_count: Generate::generate(rng),
            favourite_mapset_count: Generate::generate(rng),
            follower_count: Generate::generate(rng),
            graveyard_mapset_count: Generate::generate(rng),
            groups: GenerateRange::generate_range(rng, 0..3),
            guest_mapset_count: Generate::generate(rng),
            is_admin: Generate::generate(rng),
            is_bng: Generate::generate(rng),
            is_full_bn: Generate::generate(rng),
            is_gmt: Generate::generate(rng),
            is_limited_bn: Generate::generate(rng),
            is_moderator: Generate::generate(rng),
            is_nat: Generate::generate(rng),
            is_silenced: Generate::generate(rng),
            loved_mapset_count: Generate::generate(rng),
            mapping_follower_count: Generate::generate(rng),
            monthly_playcounts: GenerateRange::generate_range(rng, 90..91),
            page: Generate::generate(rng),
            previous_usernames: None,
            rank_history: GenerateRange::generate_range(rng, 90..91),
            ranked_mapset_count: Generate::generate(rng),
            replays_watched_counts: GenerateRange::generate_range(rng, 20..91),
            scores_best_count: Generate::generate(rng),
            scores_first_count: Generate::generate(rng),
            scores_recent_count: Generate::generate(rng),
            statistics: Generate::generate(rng),
            support_level: Generate::generate(rng),
            pending_mapset_count: Generate::generate(rng),
            medals: Some(GenerateRange::generate_range(rng, 15..270)),
        }
    }
}

impl Generate for UserFull {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        let mut user = User::generate(rng);
        user.mode = GameMode::Osu;

        let tko = User {
            mode: GameMode::Taiko,
            statistics: Some(Generate::generate(rng)),
            ..user.clone()
        };

        let ctb = User {
            mode: GameMode::Catch,
            statistics: Some(Generate::generate(rng)),
            ..user.clone()
        };

        let mna = User {
            mode: GameMode::Mania,
            statistics: Some(Generate::generate(rng)),
            ..user.clone()
        };

        [user, tko, ctb, mna].into()
    }
}
