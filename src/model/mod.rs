pub use self::{
    badge::{BadgeEntry, BadgeKey, Badges, SlimBadge},
    progress::{Finish, Progress},
    ranking::RankingUser,
    rarity::{MedalRarities, MedalRarityEntry},
    scrap::{ScrapedMedal, ScrapedUser},
    user::{OsuUser, UserFull},
};

mod badge;
mod progress;
mod ranking;
mod rarity;
mod scrap;
mod user;
