use std::collections::HashSet;

use eyre::{Context as _, Result};
use rosu_v2::prelude::GameMode;

use crate::model::{IntHasher, UserFull};

use super::Context;

impl Context {
    pub async fn get_user(&self, user_id: u32) -> Result<UserFull> {
        let osu = self.osu.user(user_id).mode(GameMode::Osu);
        let tko = self.osu.user(user_id).mode(GameMode::Taiko);
        let ctb = self.osu.user(user_id).mode(GameMode::Catch);
        let mna = self.osu.user(user_id).mode(GameMode::Mania);

        tokio::try_join!(osu, tko, ctb, mna)
            .map(|(osu, tko, ctb, mna)| [osu, tko, ctb, mna])
            .map(UserFull::from)
            .context("failed to get user from osu!api")
    }

    /// Makes 800 requests to the osu!api, very expensive call!
    pub async fn get_leaderboard_user_ids(&self) -> Result<HashSet<u32, IntHasher>> {
        let modes = [
            GameMode::Osu,
            GameMode::Taiko,
            GameMode::Catch,
            GameMode::Mania,
        ];

        let mut user_ids = HashSet::with_capacity_and_hasher(30_000, IntHasher);

        for mode in modes {
            for page in 1..=200 {
                let rankings = self
                    .osu
                    .performance_rankings(mode)
                    .page(page)
                    .await
                    .with_context(|| {
                        format!("failed to retrieve leaderboard page {page} for {mode:?}")
                    })?;

                user_ids.extend(rankings.ranking.into_iter().map(|user| user.user_id));
            }
        }

        Ok(user_ids)
    }
}
