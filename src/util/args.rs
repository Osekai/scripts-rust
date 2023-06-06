use std::{collections::HashSet, ops::BitOr};

use clap::{Parser, Subcommand};
use eyre::{Result, WrapErr};
use self_update::{backends::github::Update, Status};

use crate::task::Task;

use super::IntHasher;

pub struct Args {
    pub delay: u64,
    pub extras: HashSet<u32, IntHasher>,
    pub interval: u64,
    pub progress: bool,
    pub quiet: bool,
    pub debug: bool,
}

pub enum ArgsResult {
    Args(Args, Option<Task>),
    Update(Result<Status>),
}

impl Args {
    pub fn parse() -> ArgsResult {
        let ArgsCli {
            extra,
            interval,
            initial_delay,
            progress,
            quiet,
            debug,
            task,
            command,
        } = ArgsCli::parse();

        if let Some(ArgCommand::Update) = command {
            return ArgsResult::Update(update());
        }

        let task = task.into_iter().reduce(Task::bitor);

        // Default delay when looping is 1 minute, otherwise 0
        let delay = initial_delay.unwrap_or_else(|| task.is_none() as u64);

        let args = Args {
            delay,
            extras: extra.into_iter().collect(),
            interval,
            progress,
            quiet,
            debug,
        };

        ArgsResult::Args(args, task)
    }
}

#[derive(Parser)]
#[command(author, about = DESCRIPTION)]
struct ArgsCli {
    #[arg(short, long, value_name = "USER_ID")]
    /// Additional user id to check (repeatable)
    extra: Vec<u32>,
    #[arg(short, long, default_value_t = 12, value_name = "HOURS")]
    /// Time inbetween two tasks
    interval: u64,
    #[arg(long, value_name = "MINUTES")]
    /// Time until the first task is started
    initial_delay: Option<u64>,
    #[arg(short, long, action)]
    /// Set this if progression should be sent to osekai
    progress: bool,
    #[arg(short, long, action)]
    /// Set this if no logs should be displayed
    quiet: bool,
    #[arg(long, action)]
    /// Set this to process only one user
    debug: bool,
    #[arg(short, long)]
    /// Specific task to be run only once (repeatable)
    task: Vec<Task>,
    #[command(subcommand)]
    command: Option<ArgCommand>,
}

#[derive(Subcommand)]
enum ArgCommand {
    /// Just check for an update and install it
    Update,
}

fn update() -> Result<Status> {
    #[cfg(target_os = "windows")]
    let target = "x86_64-pc-windows-gnu";

    #[cfg(target_os = "linux")]
    let target = "x86_64-unknown-linux-musl";

    Update::configure()
        .repo_owner("Osekai")
        .repo_name("scripts-rust")
        .bin_name("osekai-scripts")
        .show_download_progress(true)
        .show_output(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .no_confirm(true)
        .target(target)
        .build()
        .wrap_err("Failed to build update")?
        .update()
        .wrap_err("Failed to apply update")
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
  - rarity: Retrieve the top 10,000 users for all modes to
      calculate medal rarities and upload them.
  - ranking: Process all users and upload them.
  - badges: Collect badges of all available users and upload them.
  - default: medals | ranking | badges
  - full: medals | ranking | badges | rarity"#;
