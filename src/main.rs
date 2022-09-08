#![deny(clippy::all, nonstandard_style, rust_2018_idioms, warnings)]
#![allow(unused)] // TODO: remove
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
    #[clap(short, long, value_parser, default_value_t = 12)]
    /// Hours inbetween two tasks
    interval: u64,
    #[clap(long, value_parser)]
    /// Minutes until the first task is started
    initial_delay: Option<u64>,
    #[clap(short, long, value_parser)]
    /// Specific task to be run only once
    task: Option<Task>,
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
    config::init().context("failed to initialize config")?;
    let args = Args::parse();

    let ctx = Context::new().await.context("failed to create context")?;

    if let Some(task) = args.task {
        let delay = args.initial_delay.unwrap_or(0);

        ctx.run_once(task, delay).await;
    } else {
        tokio::select! {
            _ = ctx.loop_forever(args) => unreachable!(),
            res = signal::ctrl_c() => match res {
                Ok(_) => info!("Received Ctrl+C"),
                Err(err) => error!("{:?}", Report::new(err).wrap_err("Failed to await ctrl+c")),
            }
        }
    }

    info!("Shutting down");

    Ok(())
}
