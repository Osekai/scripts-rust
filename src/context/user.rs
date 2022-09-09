use std::collections::HashSet;

use eyre::{Context as _, Report, Result};
use rosu_v2::prelude::{GameMode, OsuError};

use crate::{
    model::UserFull,
    util::{Eta, IntHasher},
};

use super::Context;

impl Context {
    /// Request user data of a user for all four modes
    pub async fn request_osu_user(&self, user_id: u32) -> Result<UserFull, OsuError> {
        let osu = self.osu.user(user_id).mode(GameMode::Osu);
        let tko = self.osu.user(user_id).mode(GameMode::Taiko);
        let ctb = self.osu.user(user_id).mode(GameMode::Catch);
        let mna = self.osu.user(user_id).mode(GameMode::Mania);

        tokio::try_join!(osu, tko, ctb, mna)
            .map(|(osu, tko, ctb, mna)| [osu, tko, ctb, mna])
            .map(UserFull::from)
    }

    /// Request leaderboard pages for all four modes and collect user ids.
    ///
    /// If `max_page` is not set, it defauls to 200 i.e. all pages.
    pub async fn request_leaderboards(
        &self,
        user_ids: &mut HashSet<u32, IntHasher>,
        max_page: Option<u32>,
    ) {
        info!("Requesting all leaderboard pages of all modes...");

        let modes = [
            GameMode::Osu,
            GameMode::Taiko,
            GameMode::Catch,
            GameMode::Mania,
        ];

        let max_page = max_page.map_or(200, |page| page.min(200));
        user_ids.reserve(20_000);
        let mut eta = Eta::default();

        for (mode, i) in modes.into_iter().zip(0..) {
            for page in 1..=max_page {
                let page_fut = self.osu.performance_rankings(mode).page(page);

                let rankings = match page_fut.await {
                    Ok(rankings) => rankings,
                    Err(err) => {
                        let wrap =
                            format!("failed to retrieve leaderboard page {page} for {mode:?}");
                        error!("{:?}", Report::from(err).wrap_err(wrap));

                        continue;
                    }
                };

                user_ids.extend(rankings.ranking.into_iter().map(|user| user.user_id));
                eta.tick();

                if page % 50 == 0 {
                    let curr = i * 200 + page;
                    info!(
                        "Leaderboard progress: {curr}/{max_page} | Remaining: {}",
                        eta.estimate((max_page - curr) as usize),
                    );
                }
            }

            info!("Finished requesting all leaderboard pages for {mode:?}");
        }
    }

    /// Request all user ids stored by osekai
    pub async fn request_osekai_users(&self) -> Result<HashSet<u32, IntHasher>> {
        let bytes = self
            .client
            .get_osekai_members()
            .await
            .context("failed to get osekai members")?;

        serde_json::from_slice(&bytes).with_context(|| {
            let text = String::from_utf8_lossy(&bytes);

            format!("failed to deserialize osekai members: {text}")
        })
    }
}
