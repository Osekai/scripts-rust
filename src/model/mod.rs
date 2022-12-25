pub use self::{
    badge::{BadgeEntry, Badges, SlimBadge},
    progress::Progress,
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
