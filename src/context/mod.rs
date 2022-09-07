use eyre::{Context as _, Result};
use rosu_v2::Osu;

use crate::{client::Client, config::Config, model::RankingUser};

mod medal;
mod osekai;
mod user;

pub struct Context {
    client: Client,
    osu: Osu,
}

impl Context {
    pub async fn new() -> Result<Self> {
        let config = Config::get();
        let client_id = config.tokens.osu_client_id;
        let client_secret = &config.tokens.osu_client_secret;

        let osu = Osu::new(client_id, client_secret)
            .await
            .context("failed to create osu client")?;

        let client = Client::new();

        Ok(Self { client, osu })
    }

    pub async fn run(self) {
        loop {
            let all_badges = self.gather_badges().await.expect("TODO");
            let user_ids = self.gather_users().await.expect("TODO");

            let mut users = Vec::with_capacity(user_ids.len());

            for (i, user_id) in user_ids.into_iter().enumerate() {
                match self.get_user(user_id).await {
                    Ok(user) => users.push(user),
                    Err(err) => {
                        let wrap = format!("failed to request user {user_id}");
                        error!("{:?}", err.wrap_err(wrap));
                    }
                }
            }

            match self.gather_medals().await {
                Ok(medals) => {
                    // TODO: upload medals

                    let rarities = Self::calculate_medal_rarity(&users, &medals);

                    // TODO: upload rarities
                }
                Err(err) => error!("{:?}", err.wrap_err("failed to gather medals")),
            }

            match self.gather_rarities().await {
                Ok(rarities) => {
                    let users: Vec<_> = users
                        .into_iter()
                        .map(|user| RankingUser::new(user, &rarities))
                        .collect();

                    // TODO: upload users
                }
                Err(err) => error!("{:?}", err.wrap_err("failed to gather rarities")),
            }

            // TODO: badges
        }
    }
}
