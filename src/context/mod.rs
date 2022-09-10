use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use eyre::{Context as _, Report, Result};
use rosu_v2::prelude::OsuError;
use rosu_v2::Osu;
use tokio::time::{interval, sleep};

use crate::{
    client::Client,
    config::Config,
    model::{Badges, Progress, RankingUser, ScrapedMedal, UserFull},
    task::Task,
    util::{Eta, IntHasher, TimeEstimate},
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

        let osu = Osu::builder()
            .client_id(config.tokens.osu_client_id)
            .client_secret(&config.tokens.osu_client_secret)
            .ratelimit(10)
            .build()
            .await
            .context("failed to create osu client")?;

        let client = Client::new();

        Ok(Self { client, osu })
    }

    /// Runs one iteration and then returns
    pub async fn run_once(self, task: Task, args: Args) {
        log_args_delay(Some(task), &args).await;
        let start = Instant::now();

        self.iteration(task, &args).await;

        let elapsed = TimeEstimate::new(start.elapsed());
        info!("Finished task `{task}` in {elapsed}");
    }

    /// Runs forever based on the schedule in the .env file
    pub async fn loop_forever(self, args: Args) {
        let schedule = &Config::get().schedule;

        info!("Schedule:");

        for (task, i) in schedule.iter().zip(1..) {
            info!("  {i}. {task}");
        }

        info!("");

        log_args_delay(None, &args).await;

        info!("First task starting now...");

        let duration = Duration::from_secs(args.interval * 60 * 60);
        let mut interval = interval(duration);

        for &task in schedule.iter().cycle() {
            interval.tick().await;
            let start = Instant::now();

            self.iteration(task, &args).await;

            let elapsed = start.elapsed();
            let next = interval.period() - elapsed;
            let hours = (next.as_secs() as f64) / 3600.0;
            info!("Finished task `{task}` in {}", TimeEstimate::new(elapsed));
            info!("Next task starts in {hours:.3} hour(s)");
        }
    }

    /// Runs one single iteration based on the task
    async fn iteration(&self, task: Task, args: &Args) {
        info!("Starting task `{task}`");

        let (users, badges) = self.gather_users_and_badges(task, args).await;

        // Upload badges if required
        if !badges.is_empty() && task.badges() {
            match self.client.upload_badges(&badges).await {
                Ok(_) => info!("Successfully uploaded {} badges", badges.len()),
                Err(err) => error!("{:?}", err.wrap_err("Failed to upload badges")),
            }
        }

        // If badges are all that was required then we're already done
        if task != Task::BADGES {
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
        }

        // Notify osekai that we're done uploading
        match self.client.finish_uploading(task).await {
            Ok(_) => info!("Successfully finished uploading"),
            Err(err) => error!("{:?}", err.wrap_err("Failed to finish uploading")),
        }
    }

    #[cfg(not(feature = "generate"))]
    async fn gather_users_and_badges(&self, task: Task, args: &Args) -> (Vec<UserFull>, Badges) {
        // If medals are the only thing that should be updated, requesting users is not necessary
        let mut user_ids = if task != Task::MEDALS {
            // Otherwise request the user ids stored by osekai
            match self.request_osekai_users().await {
                Ok(users) => users,
                Err(err) => {
                    error!("{:?}", err.wrap_err("Failed to request osekai users"));

                    HashSet::with_hasher(IntHasher)
                }
            }
        } else {
            HashSet::with_hasher(IntHasher)
        };

        // Retrieve users from the leaderboards if necessary
        let pages = if task.rarity() {
            Some(200)
        } else if task.ranking() {
            Some(5)
        } else {
            None
        };

        if let Some(pages) = pages {
            self.request_leaderboards(&mut user_ids, pages).await;
        }

        // In case additional user ids were given through CLI, add them here
        user_ids.extend(&args.extras);

        // Request badges stored by osekai so we know their ID and can extend the users
        let (check_badges, stored_badges) = if task.badges() {
            match self.request_osekai_badges().await {
                Ok(badges) => (true, badges),
                Err(err) => {
                    error!("{:?}", err.wrap_err("Failed to get osekai badges"));

                    (false, Vec::new())
                }
            }
        } else {
            (false, Vec::new())
        };

        let len = user_ids.len();
        let mut users = Vec::with_capacity(len);
        let mut badges = Badges::with_capacity(10_000);
        let mut eta = Eta::default();

        info!("Requesting {len} users...");

        // Request osu! user data for all users for all modes.
        // The core loop and very expensive.
        for (user_id, i) in user_ids.into_iter().zip(1..) {
            let mut user = match self.request_osu_user(user_id).await {
                Ok(user) => user,
                Err(OsuError::NotFound) => {
                    warn!("User {user_id} was not found");

                    continue;
                }
                Err(err) => {
                    let wrap = format!("Failed to request user {user_id} from osu!api");
                    error!("{:?}", Report::from(err).wrap_err(wrap));

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
            eta.tick();

            if i % 100 == 0 {
                let remaining = eta.estimate(len - i);
                info!("User progress: {i}/{len} | ETA: {remaining}");

                if args.progress {
                    let progress = Progress::new(i, len, remaining);

                    if let Err(err) = self.client.upload_progress(&progress).await {
                        error!("{:?}", err.wrap_err("Failed to upload progress"));
                    }
                }
            }
        }

        info!("Finished requesting {len} users");

        if check_badges {
            for (description, badge) in badges.iter_mut() {
                let slim_badge = stored_badges
                    .binary_search_by(|probe| probe.description.cmp(description))
                    .ok()
                    .and_then(|idx| stored_badges.get(idx));

                if let Some(slim_badge) = slim_badge {
                    badge.id = Some(slim_badge.id);
                    badge.users.extend(&slim_badge.users);
                }
            }
        }

        (users, badges)
    }

    #[cfg(feature = "generate")]
    /// Generate users with random dummy values
    async fn gather_users_and_badges(&self, task: Task, _: &Args) -> (Vec<UserFull>, Badges) {
        use rand::Rng;
        use rosu_v2::prelude::Badge;

        use crate::util::{Generate, GenerateRange};

        debug!("Start generating users...");

        let mut rng = rand::thread_rng();
        let mut users = (0..5_000).map(|_| Generate::generate(&mut rng)).collect();

        debug!("Done generating");

        let mut badges = Badges::default();

        if !task.badges() {
            return (users, badges);
        }

        let stored_badges = match self.request_osekai_badges().await {
            Ok(stored_badges) => stored_badges,
            Err(err) => {
                error!("{:?}", err.wrap_err("Failed to get osekai badges"));

                return (users, badges);
            }
        };

        for user in users.iter_mut() {
            let user_id = user.user_id();

            let badges_count = rng.gen_range(0..20);

            for _ in 0..badges_count {
                // Generate a new badge
                if rng.gen_bool(0.0001) {
                    let name = String::generate_range(&mut rng, 3..12);
                    let image_url = format!("https://www.google.com/{name}.png");

                    let mut badge = Badge {
                        awarded_at: Generate::generate(&mut rng),
                        description: GenerateRange::generate_range(&mut rng, 5..20),
                        image_url,
                        url: String::new(),
                    };

                    badges.insert(user_id, &mut badge);
                } else {
                    // Use one of the stored badges
                    let stored_badge_idx = rng.gen_range(0..stored_badges.len());
                    let stored_badge = &stored_badges[stored_badge_idx];

                    let mut badge = Badge {
                        awarded_at: Generate::generate(&mut rng),
                        description: stored_badge.description.clone(),
                        image_url: stored_badge.image_url.clone(),
                        url: String::new(),
                    };

                    badges.insert(user_id, &mut badge);
                    badges.get_mut(&stored_badge.description).id = Some(stored_badge.id);
                }
            }
        }

        (users, badges)
    }

    async fn handle_rarities_and_ranking(
        &self,
        task: Task,
        #[allow(unused_mut)] mut users: Vec<UserFull>,
        medals: &[ScrapedMedal],
    ) {
        let rarities = if users.is_empty() {
            return;
        } else if task.rarity() {
            // Leaderboard users were gathered so we can calculate proper rarities

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
        } else if task.ranking() {
            // Only osekai users were retrieved, dont calculate rarities
            // and instead just request them from osekai
            match self.request_osekai_rarities().await {
                Ok(rarities) => rarities,
                Err(err) => {
                    let err = err.wrap_err("Failed to request osekai medal rarities");

                    return error!("{err:?}");
                }
            }
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

async fn log_args_delay(task: Option<Task>, args: &Args) {
    let Args {
        delay,
        extras,
        interval,
        progress,
        ..
    } = args;

    info!("Arguments:");

    if let Some(task) = task {
        info!("  - Run a single task: {task}");
        info!("  - The task will start in {delay} minute(s)");
    } else {
        info!("  - The first task will start in {delay} minute(s)");
        info!("  - Tasks will start {interval} hour(s) after each other");
    }

    info!("  - Send progress to osekai while requesting users: {progress}");
    info!("  - Additional user ids: {extras:?}");
    info!("");

    if args.delay > 0 {
        sleep(Duration::from_secs(delay * 60)).await;
    }
}
