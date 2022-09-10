use std::collections::{BinaryHeap, HashSet};

use eyre::{Context as _, Report, Result};
use rosu_v2::prelude::{GameMode, OsuError};

use crate::{
    model::{SlimBadge, UserFull},
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
    pub async fn request_leaderboards(
        &self,
        user_ids: &mut HashSet<u32, IntHasher>,
        max_page: u32,
    ) {
        info!("Requesting the first {max_page} leaderboard pages for all modes...");

        let modes = [
            GameMode::Osu,
            GameMode::Taiko,
            GameMode::Catch,
            GameMode::Mania,
        ];

        let max_page = max_page.min(200);
        user_ids.reserve(4 * 40 * max_page as usize);
        let mut eta = Eta::default();

        for (mode, i) in modes.into_iter().zip(0..) {
            for page in 1..=max_page {
                let page_fut = self.osu.performance_rankings(mode).page(page);

                let rankings = match page_fut.await {
                    Ok(rankings) => rankings,
                    Err(err) => {
                        let wrap =
                            format!("Failed to retrieve leaderboard page {page} for {mode:?}");
                        error!("{:?}", Report::from(err).wrap_err(wrap));

                        continue;
                    }
                };

                user_ids.extend(rankings.ranking.into_iter().map(|user| user.user_id));
                eta.tick();

                if page % 50 == 0 {
                    let curr = i * 200 + page;
                    let max = max_page * 4;

                    info!(
                        "Leaderboard progress: {curr}/{max} | ETA: {}",
                        eta.estimate((max - curr) as usize),
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

    /// Request all badges stored by osekai.
    ///
    /// The resulting badges will be sorted by their description.
    pub async fn request_osekai_badges(&self) -> Result<Vec<SlimBadge>> {
        let bytes = self
            .client
            .get_osekai_badges()
            .await
            .context("failed to get osekai badges")?;

        serde_json::from_slice(&bytes)
            // Better to deserialize into a Vec and sort afterwards?
            // TODO: benchmark
            .map(BinaryHeap::into_sorted_vec)
            .with_context(|| {
                let text = String::from_utf8_lossy(&bytes);

                format!("failed to deserialize osekai badges: {text}")
            })
    }
}
