#![deny(clippy::all, nonstandard_style, rust_2018_idioms, warnings)]
#![allow(unused)] // TODO: remove

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

use context::Context;
use eyre::{Context as _, Result};
use tokio::runtime::Builder as RuntimeBuilder;

mod client;
mod config;
mod context;
mod logging;
mod model;

fn main() {
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    dotenv::dotenv().expect("failed to parse .env");
    let _log_worker_guard = logging::init();

    if let Err(err) = runtime.block_on(async_main()) {
        error!("{:?}", err.wrap_err("critical error in main"));
    }
}

async fn async_main() -> Result<()> {
    config::init().context("failed to initialize config")?;

    let ctx = Context::new().await.context("failed to create context")?;

    // Never stops
    ctx.run().await;

    info!("Shutting down");

    Ok(())
}
