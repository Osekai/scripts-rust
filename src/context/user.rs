use std::collections::HashSet;

use eyre::Report;
use rosu_v2::{
    prelude::{GameMode, OsuError, Rankings},
    OsuResult,
};

use crate::{
    model::{OsuUser, UserFull},
    util::{Eta, IntHasher},
};

use super::Context;

impl Context {
    /// Request user data of a user for all four modes
    pub async fn request_osu_user(&self, user_id: u32) -> OsuResult<OsuUser> {
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
                    Err(OsuError::NotFound) => return Ok(OsuUser::Restricted { user_id }),
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

        Ok(OsuUser::Available(UserFull::new(std, tko, ctb, mna)))
    }

    /// Request leaderboard pages for all four modes and collect user ids.
    pub async fn request_leaderboards(
        &self,
        user_ids: &mut HashSet<u32, IntHasher>,
        max_page: usize,
    ) {
        let max_page = max_page.min(200);
        user_ids.reserve(4 * 40 * max_page);
        let mut eta = Eta::default();

        info!("Requesting the first {max_page} leaderboard pages for all modes...");

        for page in 1..=max_page as u32 {
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
                let estimate = eta.estimate(max_page - page as usize);
                info!("Leaderboard progress: {page}/{max_page} | ETA: {estimate}");
            }
        }

        info!("Finished requesting {max_page} leaderboard pages for all modes");
    }
}
