#![deny(clippy::all, nonstandard_style, rust_2018_idioms, warnings)]
#![allow(unused)] // TODO: remove

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

use config::Config;
use eyre::{Context as _, Result};
use tokio::runtime::Builder as RuntimeBuilder;

mod client;
mod config;
mod logging;
mod model;

fn main() {
    let runtime = RuntimeBuilder::new_multi_thread()
        .enable_all()
        .thread_stack_size(2 * 1024 * 1024)
        .build()
        .expect("failed to build runtime");

    if let Err(report) = runtime.block_on(async_main()) {
        error!("{:?}", report.wrap_err("critical error in main"));
    }
}

async fn async_main() -> Result<()> {
    dotenv::dotenv().expect("failed to parse .env");
    let _log_worker_guard = logging::initialize();
    Config::init().context("failed to initialize config")?;

    todo!()
}
