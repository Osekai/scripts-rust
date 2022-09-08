#![deny(clippy::all, nonstandard_style, rust_2018_idioms, warnings)]
#![allow(unused)] // TODO: remove

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

use clap::Parser;
use eyre::{Context as _, Report, Result};
use tokio::{runtime::Builder as RuntimeBuilder, signal};

use self::context::Context;

mod client;
mod config;
mod context;
mod logging;
mod model;

#[derive(Parser)]
#[clap(author, about = DESCRIPTION)]
/// Script to request data, process it, and then upload
/// it to osekai in regular intervals.
pub struct Args {
    #[clap(long, value_parser, default_value_t = 12)]
    /// Hours inbetween two tasks
    task_interval: u64,
    #[clap(long, value_parser, default_value_t = 1)]
    /// Minutes until the first task is started
    initial_delay: u64,
}

fn main() {
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    dotenv::dotenv().expect("failed to parse .env");
    let _log_worker_guard = logging::init();

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
process it, and upload it to osekai."#;

async fn async_main() -> Result<()> {
    let args = Args::parse();

    DESCRIPTION.lines().for_each(|line| info!("{line}"));

    info!("");
    info!("Arguments:");
    info!(
        "  - The first task will start in {} minute(s)",
        args.initial_delay
    );
    info!(
        "  - Tasks will start {} hour(s) after each other",
        args.task_interval
    );
    info!("-------------------------------------------------");

    config::init().context("failed to initialize config")?;

    let ctx = Context::new().await.context("failed to create context")?;

    tokio::select! {
        _ = ctx.run(args) => unreachable!(),
        res = signal::ctrl_c() => match res {
            Ok(_) => info!("Received Ctrl+C"),
            Err(err) => error!("{:?}", Report::new(err).wrap_err("Failed to await ctrl+c")),
        }
    }

    info!("Shutting down");

    Ok(())
}
