use std::{collections::HashSet, mem};

use eyre::{Context as _, Result};
use rosu_v2::Osu;

use crate::{
    client::Client,
    config::Config,
    model::{Badge, RankingUser},
};

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
            let mut user_ids = match self.get_leaderboard_user_ids().await {
                Ok(user_ids) => user_ids,
                Err(err) => {
                    error!("{:?}", err.wrap_err("failed to get leaderboard users"));

                    HashSet::new()
                }
            };

            if let Err(err) = self.gather_more_users(&mut user_ids).await {
                error!("{:?}", err.wrap_err("failed to gather more users"));
            }

            let (all_badges, check_badges) = match self.gather_badges().await {
                Ok(badges) => (badges, true),
                Err(err) => {
                    error!("{:?}", err.wrap_err("failed to gather badges"));

                    (Vec::new(), false)
                }
            };

            let mut users = Vec::with_capacity(user_ids.len());
            let mut new_badges: Vec<Badge> = Vec::new();

            // 4 requests per user, potentially very expensive loop
            for (i, user_id) in user_ids.into_iter().enumerate() {
                let mut user = match self.get_user(user_id).await {
                    Ok(user) => user,
                    Err(err) => {
                        let wrap = format!("failed to request user {user_id}");
                        error!("{:?}", err.wrap_err(wrap));

                        continue;
                    }
                };

                if let Some(user_badges) = user.badges_mut().filter(|_| check_badges) {
                    for user_badge in user_badges {
                        // Skip if the badge is already known as well as the fact that the user owns it
                        let already_known = all_badges
                            .iter()
                            .find(|badge| badge.description == user_badge.description)
                            .filter(|badge| badge.users.contains(&user_id))
                            .is_some();

                        // Skip if the badge was already pushed to new_badges
                        let already_added = new_badges
                            .iter_mut()
                            .find(|badge| badge.description == user_badge.description)
                            .map(|badge| {
                                if badge.awarded_at > user_badge.awarded_at {
                                    badge.awarded_at = user_badge.awarded_at;
                                }

                                badge.users.push(user_id);
                            })
                            .is_some();

                        if !(already_known || already_added) {
                            let badge = Badge {
                                users: vec![user_id],
                                awarded_at: user_badge.awarded_at,
                                description: mem::take(&mut user_badge.description),
                                image_url: mem::take(&mut user_badge.image_url),
                                url: mem::take(&mut user_badge.url),
                            };

                            new_badges.push(badge);
                        }
                    }
                }

                users.push(user);
            }

            if !new_badges.is_empty() {
                match self.client.upload_badges(&new_badges).await {
                    Ok(_) => info!("Successfully uploaded {} badges", new_badges.len()),
                    Err(err) => error!("{:?}", err.wrap_err("failed to upload badges")),
                }
            }

            match self.gather_medals().await {
                Ok(medals) => {
                    match self.client.upload_medals(&medals).await {
                        Ok(_) => info!("Successfully uploaded {} medals", medals.len()),
                        Err(err) => error!("{:?}", err.wrap_err("failed to upload medals")),
                    }

                    let rarities = Self::calculate_medal_rarity(&users, &medals);

                    match self.client.upload_rarity(&rarities).await {
                        Ok(_) => info!("Successfully uploaded {} medal rarities", rarities.len()),
                        Err(err) => error!("{:?}", err.wrap_err("failed to upload medal rarities")),
                    }
                }
                Err(err) => error!("{:?}", err.wrap_err("failed to gather medals")),
            }

            match self.gather_rarities().await {
                Ok(rarities) => {
                    let ranking: Vec<_> = users
                        .into_iter()
                        .map(|user| RankingUser::new(user, &rarities))
                        .collect();

                    match self.client.upload_ranking(&ranking).await {
                        Ok(_) => info!("Successfully uploaded {} ranking entries", ranking.len()),
                        Err(err) => error!("{:?}", err.wrap_err("failed to upload ranking")),
                    }
                }
                Err(err) => error!("{:?}", err.wrap_err("failed to gather rarities")),
            }

            match self.client.finish_uploading().await {
                Ok(_) => info!("Successfully finished uploading"),
                Err(err) => error!("{:?}", err.wrap_err("failed to finish uploading")),
            }
        }
    }
}
