pub use self::{
    badge::{BadgeAwards, BadgeEntry, BadgeId, BadgeKey, Badges, SlimBadge, UserId},
    progress::{Finish, Progress},
    ranking::{RankingUser, RankingsIter},
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
