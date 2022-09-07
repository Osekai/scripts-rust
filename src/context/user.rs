use eyre::{Context as _, Result};
use rosu_v2::prelude::GameMode;

use crate::model::UserFull;

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
}
