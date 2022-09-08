use std::{
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
    time::{Duration, Instant},
};

use eyre::{Context as _, Result};
use rosu_v2::Osu;
use tokio::time::{interval, sleep};

use crate::{
    client::Client,
    config::Config,
    model::{Badge, RankingUser},
    task::Task,
    util::IntHasher,
    Args, DESCRIPTION,
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

    pub async fn run_once(self, task: Task) {
        info!("Arguments:");
        info!("  - Run a single task: {task}");
        info!("");

        self.iteration(task).await;

        info!("Finished task {task}");
    }

    pub async fn loop_forever(self, args: Args) {
        let schedule = &Config::get().schedule;

        info!("Schedule:");

        for (task, i) in schedule.iter().zip(1..) {
            info!("  {i}. {task}");
        }

        info!("");
        info!("Arguments:");
        info!(
            "  - The first task will start in {} minute(s)",
            args.initial_delay
        );
        info!(
            "  - Tasks will start {} hour(s) after each other",
            args.interval
        );
        info!("");

        if args.initial_delay > 0 {
            let duration = Duration::from_secs(args.initial_delay * 60);
            sleep(duration).await;
        }

        info!("First task starting now...");

        let duration = Duration::from_secs(args.interval * 60 * 60);
        let mut interval = interval(duration);

        for &task in schedule.iter().cycle() {
            interval.tick().await;
            let start = Instant::now();

            self.iteration(task).await;

            let end = Instant::now();
            let next = interval.period() - (end - start);
            let hours = (next.as_secs() as f64) / 3600.0;
            info!("Next task starts in {hours:.3} hour(s)");
        }
    }

    async fn iteration(&self, task: Task) {
        info!("Starting task `{task}`");

        let mut user_ids = if task.leaderboard() {
            match self.get_leaderboard_user_ids().await {
                Ok(user_ids) => user_ids,
                Err(err) => {
                    error!("{:?}", err.wrap_err("Failed to get leaderboard users"));

                    HashSet::with_hasher(IntHasher)
                }
            }
        } else {
            HashSet::with_hasher(IntHasher)
        };

        if let Err(err) = self.gather_more_users(&mut user_ids).await {
            error!("{:?}", err.wrap_err("Failed to gather more users"));
        }

        let (all_badges, check_badges) = if task.badges() {
            match self.gather_badges().await {
                Ok(badges) => (badges, true),
                Err(err) => {
                    error!("{:?}", err.wrap_err("Failed to gather badges"));

                    (HashMap::new(), false)
                }
            }
        } else {
            (HashMap::new(), false)
        };

        let mut users = Vec::with_capacity(user_ids.len());
        let mut new_badges: Vec<Badge> = Vec::new();

        // 4 requests per user, potentially very expensive loop
        for (i, user_id) in user_ids.into_iter().enumerate() {
            let mut user = match self.get_user(user_id).await {
                Ok(user) => user,
                Err(err) => {
                    let wrap = format!("Failed to request user {user_id}");
                    error!("{:?}", err.wrap_err(wrap));

                    continue;
                }
            };

            if let Some(user_badges) = user.badges_mut().filter(|_| check_badges) {
                for user_badge in user_badges {
                    // Skip if the badge is already known as well as the fact that the user owns it
                    let already_known = all_badges
                        .get(&user_badge.description)
                        .filter(|badge| badge.users.contains(&user_id))
                        .is_some();

                    // Skip if the badge was already pushed to new_badges
                    let already_added = new_badges
                        .iter_mut()
                        .find(|badge| *badge.description == user_badge.description)
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
                            description: Rc::new(mem::take(&mut user_badge.description)),
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
                Err(err) => error!("{:?}", err.wrap_err("Failed to upload badges")),
            }
        }

        if task.medals() {
            match self.gather_medals().await {
                Ok(medals) => {
                    match self.client.upload_medals(&medals).await {
                        Ok(_) => info!("Successfully uploaded {} medals", medals.len()),
                        Err(err) => error!("{:?}", err.wrap_err("Failed to upload medals")),
                    }

                    if task.rarity() && !users.is_empty() {
                        let rarities = Self::calculate_medal_rarity(&users, &medals);

                        match self.client.upload_rarity(&rarities).await {
                            Ok(_) => {
                                info!("Successfully uploaded {} medal rarities", rarities.len())
                            }
                            Err(err) => {
                                error!("{:?}", err.wrap_err("Failed to upload medal rarities"))
                            }
                        }
                    }
                }
                Err(err) => error!("{:?}", err.wrap_err("Failed to gather medals")),
            }
        }

        if task.ranking() && !users.is_empty() {
            match self.gather_rarities().await {
                Ok(rarities) => {
                    let ranking: Vec<_> = users
                        .into_iter()
                        .map(|user| RankingUser::new(user, &rarities))
                        .collect();

                    match self.client.upload_ranking(&ranking).await {
                        Ok(_) => info!("Successfully uploaded {} ranking entries", ranking.len()),
                        Err(err) => error!("{:?}", err.wrap_err("Failed to upload ranking")),
                    }
                }
                Err(err) => error!("{:?}", err.wrap_err("Failed to gather rarities")),
            }
        }

        match self.client.finish_uploading().await {
            Ok(_) => info!("Successfully finished uploading"),
            Err(err) => error!("{:?}", err.wrap_err("Failed to finish uploading")),
        }
    }
}
