use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use eyre::{Context as _, Result};
use rosu_v2::Osu;
use tokio::time::{interval, sleep};

use crate::{
    client::Client,
    config::Config,
    model::{Badges, RankingUser, ScrapedMedal, UserFull},
    task::Task,
    util::IntHasher,
    Args,
};

mod medal;
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

    /// Runs one iteration and then returns
    pub async fn run_once(self, task: Task, delay: u64, extras: &[u32]) {
        info!("Arguments:");
        info!("  - Run a single task: {task}");
        info!("  - The task will start in {delay} minute(s)");
        info!("");

        if delay > 0 {
            let duration = Duration::from_secs(delay * 60);
            sleep(duration).await;
        }

        self.iteration(task, extras).await;

        info!("Finished task {task}");
    }

    /// Runs forever based on the schedule in the .env file
    pub async fn loop_forever(self, args: Args) {
        let schedule = &Config::get().schedule;
        let delay = args.initial_delay.unwrap_or(1);

        info!("Schedule:");

        for (task, i) in schedule.iter().zip(1..) {
            info!("  {i}. {task}");
        }

        info!("");
        info!("Arguments:");
        info!("  - The first task will start in {delay} minute(s)");
        info!(
            "  - Tasks will start {} hour(s) after each other",
            args.interval
        );
        info!("");

        if delay > 0 {
            let duration = Duration::from_secs(delay * 60);
            sleep(duration).await;
        }

        info!("First task starting now...");

        let duration = Duration::from_secs(args.interval * 60 * 60);
        let mut interval = interval(duration);

        for &task in schedule.iter().cycle() {
            interval.tick().await;
            let start = Instant::now();

            self.iteration(task, &args.extra).await;

            let end = Instant::now();
            let next = interval.period() - (end - start);
            let hours = (next.as_secs() as f64) / 3600.0;
            info!("Next task starts in {hours:.3} hour(s)");
        }
    }

    /// Runs one single iteration based on the task
    async fn iteration(&self, task: Task, extras: &[u32]) {
        info!("Starting task `{task}`");

        let (users, badges) = self.gather_users_and_badges(task, extras).await;

        // Upload badges if required
        if !badges.is_empty() && task.badges() {
            match self.client.upload_badges(&badges).await {
                Ok(_) => info!("Successfully uploaded {} badges", badges.len()),
                Err(err) => error!("{:?}", err.wrap_err("Failed to upload badges")),
            }
        }

        // Request medals
        match self.request_medals().await {
            Ok(medals) => {
                // Upload medals if required
                if task.medals() {
                    match self.client.upload_medals(&medals).await {
                        Ok(_) => info!("Successfully uploaded {} medals", medals.len()),
                        Err(err) => error!("{:?}", err.wrap_err("Failed to upload medals")),
                    }
                }

                self.handle_rarities_and_ranking(task, users, &medals).await;
            }
            Err(err) => error!("{:?}", err.wrap_err("Failed to gather medals")),
        }

        // Notify osekai that we're done uploading
        match self.client.finish_uploading().await {
            Ok(_) => info!("Successfully finished uploading"),
            Err(err) => error!("{:?}", err.wrap_err("Failed to finish uploading")),
        }
    }

    #[cfg(not(feature = "generate"))]
    async fn gather_users_and_badges(&self, task: Task, extras: &[u32]) -> (Vec<UserFull>, Badges) {
        // Retrieve users from the leaderboards if necessary, otherwise start blank
        let mut user_ids = if task.leaderboard() {
            match self.request_leaderboards().await {
                Ok(user_ids) => user_ids,
                Err(err) => {
                    error!("{:?}", err.wrap_err("Failed to get leaderboard users"));

                    HashSet::with_hasher(IntHasher)
                }
            }
        } else {
            HashSet::with_hasher(IntHasher)
        };

        // If medals are the only thing that should be updated, requesting users is not necessary
        if task != Task::MEDALS {
            // Otherwise request the user ids stored by osekai
            if let Err(err) = self.request_osekai_users(&mut user_ids).await {
                error!("{:?}", err.wrap_err("Failed to gather more users"));
            }
        }

        // In case additional user ids were given through CLI, add them here
        user_ids.extend(extras);

        let check_badges = task.badges();
        let len = user_ids.len();
        let mut users = Vec::with_capacity(len);
        let mut badges = Badges::with_capacity(10_000);

        info!("Requesting {len} users...");

        // Request osu! user data for all users for all modes.
        // The core loop and very expensive
        for (user_id, i) in user_ids.into_iter().zip(1..) {
            let mut user = match self.request_osu_user(user_id).await {
                Ok(user) => user,
                Err(err) => {
                    let wrap = format!("Failed to request user {user_id}");
                    error!("{:?}", err.wrap_err(wrap));

                    continue;
                }
            };

            // Process badges if required
            if let Some(user_badges) = user.badges_mut().filter(|_| check_badges) {
                for badge in user_badges.iter_mut() {
                    badges.insert(user_id, badge);
                }
            }

            users.push(user);

            if i % 1000 == 0 {
                info!("User progress: {i}/{len}");
            }
        }

        (users, badges)
    }

    #[cfg(feature = "generate")]
    /// Generate users with random dummy values
    async fn gather_users_and_badges(&self, _: Task, _: &[u32]) -> (Vec<UserFull>, Badges) {
        debug!("Start generating users...");

        let mut rng = rand::thread_rng();

        let users: Vec<UserFull> = (0..5_000)
            .map(|_| crate::util::Generate::generate(&mut rng))
            .collect();

        debug!("Done generating");

        (users, Badges::default())
    }

    async fn handle_rarities_and_ranking(
        &self,
        task: Task,
        #[allow(unused_mut)] mut users: Vec<UserFull>,
        medals: &[ScrapedMedal],
    ) {
        // If rarities are required, calculate them, otherwise just return
        let rarities = if !users.is_empty() && (task.rarity() || task.ranking()) {
            #[cfg(feature = "generate")]
            {
                // Make sure that all medal ids are valid if they were randomly generated
                let medal_ids: HashSet<_, IntHasher> =
                    medals.iter().map(|medal| medal.id).collect();

                for user in users.iter_mut() {
                    if let Some(medals) = user.medals_mut() {
                        medals.retain(|medal| medal_ids.contains(&medal.medal_id));
                    }
                }
            }

            Self::calculate_rarities(&users, medals)
        } else {
            return;
        };

        // Upload rarities if required
        if task.rarity() {
            match self.client.upload_rarity(&rarities).await {
                Ok(_) => info!("Successfully uploaded {} medal rarities", rarities.len()),
                Err(err) => error!("{:?}", err.wrap_err("Failed to upload medal rarities")),
            }
        }

        // Calculate and upload user rankings if required
        if task.ranking() {
            let ranking: Vec<_> = users
                .into_iter()
                .map(|user| RankingUser::new(user, &rarities))
                .collect();

            match self.client.upload_ranking(&ranking).await {
                Ok(_) => info!("Successfully uploaded {} ranking entries", ranking.len()),
                Err(err) => error!("{:?}", err.wrap_err("Failed to upload ranking")),
            }
        }
    }
}
