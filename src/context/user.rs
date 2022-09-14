use std::{
    collections::{BinaryHeap, HashSet},
    fmt::{Formatter, Result as FmtResult},
};

use eyre::{Context as _, Report, Result};
use rosu_v2::{
    prelude::{GameMode, OsuError, Rankings},
    OsuResult,
};
use serde::{
    de::{SeqAccess, Visitor},
    Deserializer as _,
};
use serde_json::{de::SliceRead, Deserializer};

use crate::{
    model::{SlimBadge, UserFull},
    util::{Eta, IntHasher},
};

use super::Context;

impl Context {
    /// Request user data of a user for all four modes
    pub async fn request_osu_user(&self, user_id: u32) -> OsuResult<UserFull> {
        let (std_res, tko_res, ctb_res, mna_res) = tokio::join!(
            self.osu.user(user_id).mode(GameMode::Osu),
            self.osu.user(user_id).mode(GameMode::Taiko),
            self.osu.user(user_id).mode(GameMode::Catch),
            self.osu.user(user_id).mode(GameMode::Mania),
        );

        macro_rules! handle_res {
            ($res:ident: $mode:path) => {
                match $res {
                    Ok(user) => user,
                    // Retry on error "http2 error: connection error received: not a result of an error"
                    // see https://github.com/hyperium/hyper/issues/2500
                    Err(OsuError::Request { source })
                        if source.message().to_string().starts_with("http2 error") =>
                    {
                        self.osu.user(user_id).mode($mode).await?
                    }
                    Err(err) => return Err(err),
                }
            };
        }

        let std = handle_res!(std_res: GameMode::Osu);
        let tko = handle_res!(tko_res: GameMode::Taiko);
        let ctb = handle_res!(ctb_res: GameMode::Catch);
        let mna = handle_res!(mna_res: GameMode::Mania);

        Ok(UserFull::new(std, tko, ctb, mna))
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
                let estimate = eta.estimate(max_page - page);
                info!("Leaderboard progress: {page}/{max_page} | ETA: {estimate}");
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

    pub async fn request_osekai_ranking(
        &self,
        user_ids: &mut HashSet<u32, IntHasher>,
    ) -> Result<()> {
        let bytes = self
            .client
            .get_osekai_ranking()
            .await
            .context("failed to get osekai ranking")?;

        Deserializer::new(SliceRead::new(&bytes))
            .deserialize_seq(UserIdVisitor(user_ids))
            .with_context(|| {
                let text = String::from_utf8_lossy(&bytes);

                format!("failed to deserialize osekai ranking: {text}")
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

struct UserIdVisitor<'u>(&'u mut HashSet<u32, IntHasher>);

impl<'de> Visitor<'de> for UserIdVisitor<'_> {
    type Value = ();

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("a list containing user ids")
    }

    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut s: A) -> Result<Self::Value, A::Error> {
        while let Some(elem) = s.next_element()? {
            self.0.insert(elem);
        }

        Ok(())
    }
}
