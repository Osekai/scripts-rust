use std::{collections::HashSet, ops::BitOr};

use clap::Parser;

use crate::task::Task;

use super::IntHasher;

pub struct Args {
    pub delay: u64,
    pub extras: HashSet<u32, IntHasher>,
    pub interval: u64,
    pub progress: bool,
    pub quiet: bool,
}

impl Args {
    pub fn parse() -> (Self, Option<Task>) {
        let ArgsCli {
            extra,
            interval,
            initial_delay,
            progress,
            quiet,
            task,
        } = ArgsCli::parse();

        let task = task.into_iter().reduce(Task::bitor);

        // Default delay when looping is 1 minute, otherwise 0
        let delay = initial_delay.unwrap_or_else(|| task.is_none() as u64);

        let args = Args {
            delay,
            extras: extra.into_iter().collect(),
            interval,
            progress,
            quiet,
        };

        (args, task)
    }
}

#[derive(Parser)]
#[clap(author, about = DESCRIPTION)]
struct ArgsCli {
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
    /// Set this if progression should be sent to osekai
    progress: bool,
    #[clap(short, long, action)]
    /// Set this if no logs should be displayed
    quiet: bool,
    #[clap(short, long)]
    /// Specific task to be run only once (repeatable)
    task: Vec<Task>,
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
