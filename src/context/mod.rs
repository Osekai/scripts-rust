use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use eyre::{Context as _, Report, Result};
use rosu_v2::Osu;
use tokio::{
    task::JoinHandle,
    time::{interval, sleep},
};

use crate::{
    client::Client,
    config::Config,
    database::Database,
    model::{Badges, MedalRarities, OsuUser, Progress, RankingsIter, ScrapedMedal},
    task::Task,
    util::{Eta, IntHasher, TimeEstimate},
    Args,
};

mod medal;
mod user;
mod webhook;

pub struct Context {
    client: Client,
    osu: Osu,
    mysql: Database,
}

impl Context {
    pub async fn new() -> Result<Self> {
        let config = Config::get();

        let osu = Osu::builder()
            .client_id(config.tokens.osu_client_id)
            .client_secret(&*config.tokens.osu_client_secret)
            .ratelimit(10)
            .build()
            .await
            .context("failed to create osu client")?;

        let client = Client::new();

        let mysql = Database::new(&config.database_url).await?;

        Ok(Self { client, osu, mysql })
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

            let next = interval
                .period()
                .checked_sub(elapsed)
                .unwrap_or(Duration::ZERO);

            let hours = (next.as_secs() as f64) / 3600.0;
            info!("Finished task `{task}` in {}", TimeEstimate::new(elapsed));
            info!("Next task starts in {hours:.3} hour(s)");
        }
    }

    /// Runs one single iteration based on the task
    async fn iteration(&self, task: Task, args: &Args) {
        info!("Starting task `{task}`");

        let mut db_handles = Vec::new();

        let (users, badges, progress) = self.gather_users_and_badges(task, args).await;

        // Store badges if required
        if !badges.is_empty() && task.badges() {
            db_handles.push(self.mysql.store_badges(badges));
        }

        // If badges are all that was required then we're already done
        if task != Task::BADGES {
            match self.request_medals().await {
                Ok(medals) => {
                    // Fetch medal ids to see if we received new ones
                    match self.mysql.fetch_medal_ids().await {
                        Ok(old_medals) => {
                            let new_medals: MedalRarities = medals
                                .iter()
                                .filter(|medal| !old_medals.contains(&medal.id))
                                .map(|medal| (medal.id, 0, 0.0))
                                .collect();

                            // If there are new medals, store their rarities
                            if !new_medals.is_empty() {
                                db_handles.push(self.mysql.store_rarities(new_medals));
                            }
                        }
                        Err(err) => error!(?err, "Failed to fetch medal ids from DB"),
                    };

                    // Store medals if required
                    if task.medals() {
                        // Note that this call needs to happen before storing
                        // rarities so that the DB table does not deadlock.
                        self.mysql.store_medals(&medals).await;
                    }

                    self.handle_rarities_and_ranking(task, users, &medals, &mut db_handles)
                        .await;
                }
                Err(err) => error!(?err, "Failed to gather medals"),
            }
        }

        for handle in db_handles {
            let _ = handle.await;
        }

        // Notify a webhook that we're done storing
        match self.handle_finish(progress.into()).await {
            Ok(_) => info!("Successfully notified webhook about finishing"),
            Err(err) => error!(?err, "Failed to notify webhook about finishing"),
        }
    }

    async fn gather_users_and_badges(
        &self,
        task: Task,
        args: &Args,
    ) -> (Vec<OsuUser>, Badges, Progress) {
        // If medals are the only thing that should be updated, fetching users is not necessary
        let mut user_ids = if task != Task::MEDALS {
            // Otherwise fetch the user ids stored by osekai
            match self.mysql.fetch_osekai_user_ids().await {
                Ok(users) => users,
                Err(err) => {
                    error!(?err, "Failed to fetch osekai user ids");

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

        if let Some(pages) = pages.filter(|_| !args.debug) {
            self.request_leaderboards(&mut user_ids, pages).await;
        }

        // If really ALL users are wanted, fetch them from osekai
        if task.contains(Task::FULL) && !args.debug {
            if let Err(err) = self.mysql.fetch_osekai_ranking_ids(&mut user_ids).await {
                error!(?err, "Failed to fetch osekai ranking ids");
            }
        }

        // In case additional user ids were given through CLI, add them here
        user_ids.extend(&args.extras);

        // Fetch badges stored by osekai so we know their ID and can extend the users
        let (check_badges, stored_badges) = if task.badges() {
            match self.mysql.fetch_badges().await {
                Ok(badges) => (true, badges),
                Err(err) => {
                    error!(?err, "Failed to fetch badges from DB");

                    (false, Badges::default())
                }
            }
        } else {
            (false, Badges::default())
        };

        if args.debug {
            user_ids = user_ids.into_iter().take(10).collect();

            if user_ids.is_empty() {
                user_ids.insert(2211396);
            }
        }

        let len = user_ids.len();
        let mut users = Vec::with_capacity(len);
        let mut eta = Eta::default();

        let badge_capacity = if check_badges { 10_000 } else { 0 };
        let mut badges_incoming = Badges::with_capacity(badge_capacity);
        let mut badge_name_buf = String::new();

        info!("Requesting {len} user(s)...");

        let mut progress = Progress::new(len, task);

        if args.progress {
            match self.handle_progress(&progress).await {
                Ok(_) => info!("Successfully handled initial progress"),
                Err(err) => error!(?err, "Failed to handle initial progress"),
            }
        }

        // Request osu! user data for all users for all modes.
        // The core loop and very expensive.
        for (user_id, i) in user_ids.into_iter().zip(1..) {
            let mut user = match self.request_osu_user(user_id).await {
                Ok(user) => user,
                Err(err) => {
                    error!(err = ?Report::new(err), "Failed to request user {user_id} from osu!api");

                    continue;
                }
            };

            // Process badges if required
            if check_badges {
                if let OsuUser::Available(ref mut user) = user {
                    for badge in user.badges.iter_mut() {
                        badges_incoming.push(user.user_id, badge, &mut badge_name_buf);
                    }
                }
            }

            users.push(user);
            eta.tick();

            if i % Progress::INTERVAL == 0 {
                let remaining_time = eta.estimate(len - i);
                info!("User progress: {i}/{len} | ETA: {remaining_time}");

                if args.progress {
                    progress.update(i, &eta);

                    match self.handle_progress(&progress).await {
                        Ok(_) => info!("Successfully handled progress"),
                        Err(err) => error!(?err, "Failed to handle progress"),
                    }
                }
            }
        }

        info!("Finished requesting {len} users");

        if args.progress {
            progress.finish();

            match self.handle_progress(&progress).await {
                Ok(_) => info!("Successfully handled final progress"),
                Err(err) => error!(?err, "Failed to handle final progress"),
            }
        }

        if check_badges {
            badges_incoming.merge(stored_badges);
        }

        (users, badges_incoming, progress)
    }

    async fn handle_rarities_and_ranking(
        &self,
        task: Task,
        users: Vec<OsuUser>,
        medals: &[ScrapedMedal],
        db_handles: &mut Vec<JoinHandle<()>>,
    ) {
        let rarities = if users.is_empty() {
            return;
        } else if task.rarity() {
            // Leaderboard users were gathered so we can calculate proper rarities
            Self::calculate_rarities(&users, medals)
        } else if task.ranking() {
            // Only osekai users were retrieved, dont calculate rarities
            // and instead just fetch them from osekai
            match self.mysql.fetch_medal_rarities().await {
                Ok(rarities) => rarities,
                Err(err) => return error!(?err, "Failed to fetch medal rarities from DB"),
            }
        } else {
            return;
        };

        // Calculate and store user rankings if required
        if task.ranking() {
            let rankings_iter = RankingsIter::new(users, rarities.clone());
            db_handles.push(self.mysql.store_rankings(rankings_iter));
        }

        // Store rarities if required
        if task.rarity() {
            db_handles.push(self.mysql.store_rarities(rarities));
        }
    }
}

async fn log_args_delay(task: Option<Task>, args: &Args) {
    let Args {
        delay,
        extras,
        interval,
        progress,
        debug: debug_, // tracing::info doesn't like variables called `debug`
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
    info!("  - Debug mode enabled: {debug_}");
    info!("");

    if args.delay > 0 {
        sleep(Duration::from_secs(delay * 60)).await;
    }
}
