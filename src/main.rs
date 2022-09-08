#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]
#![cfg_attr(feature = "generate", allow(unused))]

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

use clap::Parser;
use eyre::{Context as _, Report, Result};
use task::Task;
use tokio::{runtime::Builder as RuntimeBuilder, signal};

use self::context::Context;

mod client;
mod config;
mod context;
mod logging;
mod model;
mod schedule;
mod task;
mod util;

#[derive(Parser)]
#[clap(author, about = DESCRIPTION)]
pub struct Args {
    #[clap(short, long, value_name = "USER_ID")]
    /// Additional user id to check (repeatable)
    extra: Vec<u32>,
    #[clap(short, long, default_value_t = 12, value_name = "HOURS")]
    /// Time inbetween two tasks
    interval: u64,
    #[clap(long, value_name = "MINUTES")]
    /// Time until the first task is started
    initial_delay: Option<u64>,
    #[clap(short, long, action)]
    /// Set this if no logs should be displayed
    quiet: bool,
    #[clap(short, long)]
    /// Specific task to be run only once
    task: Option<Task>,
}

fn main() {
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    if dotenv::dotenv().is_err() {
        panic!(
            "Failed to parse .env file. \
            Be sure there is one in the same folder as this executable."
        );
    }

    if let Err(err) = runtime.block_on(async_main()) {
        error!("{:?}", err.wrap_err("Critical error in main"));
    }
}

static DESCRIPTION: &str = r#"
#################################################
##  ,-----.               ,--.           ,--.  ##
## '  .-.  ' ,---.  ,---. |  |,-. ,--,--.`--'  ##
## |  | |  |(  .-' | .-. :|     /' ,-.  |,--.  ##
## '  '-'  '.-'  `)\   --.|  \  \\ '-'  ||  |  ##
##  `-----' `----'  `----'`--'`--'`--`--'`--'  ##
#################################################

Script to gather medal, user, and badge data, 
process it, and upload it to osekai.

Task values:
  - medals: A full list of medals will be retrieved and uploaded.
  - leaderboard: In addition to osekai's users, the top 10,000
      leaderboard users for all modes will be retrieved.
  - rarity: Based on available users, medal rarities will be
      calculated and uploaded.
  - ranking: Process all users and upload them.
  - badges: Collect badges of all available users and upload them.
  - default: medals | rarity | ranking | badges
  - full: medals | rarity | ranking | badges | leaderboard"#;

async fn async_main() -> Result<()> {
    let args = Args::parse();
    let _log_worker_guard = logging::init(args.quiet);
    config::init().context("failed to initialize config")?;

    let ctx = Context::new().await.context("failed to create context")?;

    if let Some(task) = args.task {
        let delay = args.initial_delay.unwrap_or(0);

        ctx.run_once(task, delay, &args.extra).await;
    } else {
        tokio::select! {
            _ = ctx.loop_forever(args) => unreachable!(),
            res = signal::ctrl_c() => match res {
                Ok(_) => info!("Received Ctrl+C"),
                Err(err) => error!("{:?}", Report::new(err).wrap_err("Failed to await Ctrl+C")),
            }
        }
    }

    info!("Shutting down");

    Ok(())
}
