use std::{
    collections::HashSet,
    fmt::{Formatter, Result as FmtResult},
};

use eyre::{Context as _, Result};
use rosu_v2::prelude::GameMode;
use serde::{
    de::{Error as SerdeError, SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer as _,
};
use serde_json::{de::SliceRead, Deserializer};

use crate::{model::UserFull, util::IntHasher};

use super::Context;

impl Context {
    /// Request user data of a user for all four modes
    pub async fn request_osu_user(&self, user_id: u32) -> Result<UserFull> {
        let osu = self.osu.user(user_id).mode(GameMode::Osu);
        let tko = self.osu.user(user_id).mode(GameMode::Taiko);
        let ctb = self.osu.user(user_id).mode(GameMode::Catch);
        let mna = self.osu.user(user_id).mode(GameMode::Mania);

        tokio::try_join!(osu, tko, ctb, mna)
            .map(|(osu, tko, ctb, mna)| [osu, tko, ctb, mna])
            .map(UserFull::from)
            .context("failed to get user from osu!api")
    }

    /// Request all leaderboard pages for all four modes for a total of
    /// 800 requests - expensive call!
    pub async fn request_leaderboards(&self) -> Result<HashSet<u32, IntHasher>> {
        info!("Requesting all leaderboard pages of all modes...");

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

            info!("Finished requesting all leaderboard pages for {mode:?}");
        }

        Ok(user_ids)
    }

    /// Request all user ids stored by osekai
    pub async fn request_osekai_users(&self, users: &mut HashSet<u32, IntHasher>) -> Result<()> {
        let bytes = self
            .client
            .get_osekai_members()
            .await
            .context("failed to get osekai members")?;

        Deserializer::new(SliceRead::new(&bytes))
            .deserialize_seq(ExtendUsersVisitor(users))
            .with_context(|| {
                let text = String::from_utf8_lossy(&bytes);

                format!("failed to deserialize osekai members: {text}")
            })
    }
}

struct ExtendUsersVisitor<'u>(&'u mut HashSet<u32, IntHasher>);

impl<'de> Visitor<'de> for ExtendUsersVisitor<'_> {
    type Value = ();

    #[inline]
    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(r#"a list of `{"Id": "<number>"}` objects"#)
    }

    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        #[derive(Deserialize)]
        struct IdEntry<'s> {
            #[serde(rename = "Id")]
            id: &'s str,
        }

        while let Some(IdEntry { id }) = seq.next_element()? {
            let id = id
                .parse()
                .map_err(|_| SerdeError::invalid_value(Unexpected::Str(id), &"an integer"))?;

            self.0.insert(id);
        }

        Ok(())
    }
}
