#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

use eyre::{Context as _, Report, Result};
use self_update::Status;
use task::Task;
use tokio::{runtime::Builder as RuntimeBuilder, signal};
use util::ArgsResult;

use crate::util::Args;

use self::context::Context;

mod client;
mod config;
mod context;
mod logging;
mod model;
mod schedule;
mod task;
mod util;

fn main() {
    if dotenv::dotenv().is_err() {
        panic!(
            "Failed to parse .env file. \
            Be sure there is one in the same folder as this executable."
        );
    }

    // Needs to happen outside of a runtime because
    // self-updating will use its own runtime
    let (args, task) = match Args::parse() {
        ArgsResult::Args(args, task) => (args, task),
        ArgsResult::Update(res) => {
            match res {
                Ok(Status::Updated(version)) => println!("Updated to version {version}!"),
                Ok(Status::UpToDate(_)) => println!("Already up-to-date!"),
                Err(err) => eprintln!("{err:?}"),
            }

            return;
        }
    };

    let _log_worker_guard = logging::init(args.quiet);

    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    if let Err(err) = runtime.block_on(async_main(args, task)) {
        error!("{:?}", err.wrap_err("Critical error in main"));
    }
}

async fn async_main(mut args: Args, task: Option<Task>) -> Result<()> {
    config::init(&mut args).context("failed to initialize config")?;

    let ctx = Context::new().await.context("failed to create context")?;

    tokio::select! {
        _ = run(ctx, args, task) => {},
        res = signal::ctrl_c() => match res {
            Ok(_) => info!("Received Ctrl+C"),
            Err(err) => error!("{:?}", Report::new(err).wrap_err("Failed to await Ctrl+C")),
        }
    }

    info!("Shutting down");

    Ok(())
}

async fn run(ctx: Context, args: Args, task: Option<Task>) {
    if let Some(task) = task {
        ctx.run_once(task, args).await
    } else {
        ctx.loop_forever(args).await
    }
}
