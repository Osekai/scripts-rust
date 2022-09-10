use std::collections::{BinaryHeap, HashSet};

use eyre::{Context as _, Report, Result};
use rosu_v2::{
    prelude::{GameMode, Rankings},
    OsuResult,
};

use crate::{
    model::{SlimBadge, UserFull},
    util::{Eta, IntHasher},
};

use super::Context;

impl Context {
    /// Request user data of a user for all four modes
    pub async fn request_osu_user(&self, user_id: u32) -> OsuResult<UserFull> {
        tokio::try_join!(
            self.osu.user(user_id).mode(GameMode::Osu),
            self.osu.user(user_id).mode(GameMode::Taiko),
            self.osu.user(user_id).mode(GameMode::Catch),
            self.osu.user(user_id).mode(GameMode::Mania),
        )
        .map(|(std, tko, ctb, mna)| [std, tko, ctb, mna])
        .map(UserFull::from)
    }

    /// Request leaderboard pages for all four modes and collect user ids.
    pub async fn request_leaderboards(
        &self,
        user_ids: &mut HashSet<u32, IntHasher>,
        max_page: u32,
    ) {
        let max_page = max_page.min(200);
        user_ids.reserve(4 * 40 * max_page as usize);
        let mut eta = Eta::default();

        info!("Requesting the first {max_page} leaderboard pages for all modes...");

        for page in 1..=max_page {
            let std_fut = self.osu.performance_rankings(GameMode::Osu).page(page);
            let tko_fut = self.osu.performance_rankings(GameMode::Taiko).page(page);
            let ctb_fut = self.osu.performance_rankings(GameMode::Catch).page(page);
            let mna_fut = self.osu.performance_rankings(GameMode::Mania).page(page);

            let (std_res, tko_res, ctb_res, mna_res) =
                tokio::join!(std_fut, tko_fut, ctb_fut, mna_fut);

            fn extend_users(
                rankings_res: OsuResult<Rankings>,
                user_ids: &mut HashSet<u32, IntHasher>,
                page: u32,
                mode: GameMode,
            ) {
                match rankings_res {
                    Ok(rankings) => {
                        user_ids.extend(rankings.ranking.into_iter().map(|user| user.user_id))
                    }
                    Err(err) => {
                        let wrap =
                            format!("Failed to retrieve leaderboard page {page} for {mode:?}");
                        error!("{:?}", Report::from(err).wrap_err(wrap));
                    }
                }
            }

            extend_users(std_res, user_ids, page, GameMode::Osu);
            extend_users(tko_res, user_ids, page, GameMode::Taiko);
            extend_users(ctb_res, user_ids, page, GameMode::Catch);
            extend_users(mna_res, user_ids, page, GameMode::Mania);

            eta.tick();

            if page % 25 == 0 {
                info!(
                    "Leaderboard progress: {page}/{max_page} | ETA: {}",
                    eta.estimate((max_page - page) as usize),
                );
            }
        }

        info!("Finished requesting {max_page} leaderboard pages for all modes");
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
