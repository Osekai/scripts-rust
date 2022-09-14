use serde::Serialize;

use crate::{task::Task, util::TimeEstimate};

#[derive(Serialize)]
pub struct Progress {
    current: usize,
    total: usize,
    eta_seconds: Option<u64>,
    task: Task,
}

impl Progress {
    pub fn new(current: usize, total: usize, remaining: TimeEstimate, task: Task) -> Self {
        Self {
            current,
            total,
            eta_seconds: remaining.as_seconds(),
            task,
        }
    }
}
