use serde::Serialize;
use time::OffsetDateTime;

use crate::{task::Task, util::Eta};

#[derive(Serialize)]
pub struct Progress {
    #[serde(skip)]
    pub start: OffsetDateTime,
    pub current: usize,
    pub total: usize,
    pub eta_seconds: Option<u64>,
    pub task: Task,
}

impl Progress {
    pub const INTERVAL: usize = 100;

    pub fn new(total: usize, task: Task) -> Self {
        let start = OffsetDateTime::now_utc();

        Self {
            start,
            task,
            total,
            current: 0,
            eta_seconds: None,
        }
    }

    pub fn update(&mut self, current: usize, eta: &Eta) {
        self.current = current;

        let remaining = eta.estimate(self.total - current);
        self.eta_seconds = remaining.as_seconds();
    }

    pub fn finish(&mut self) {
        self.current = self.total;
        self.eta_seconds = Some(0);
    }
}

#[derive(Copy, Clone, Serialize)]
pub struct Finish {
    pub requested_users: usize,
    pub task: Task,
}

impl From<Progress> for Finish {
    fn from(progress: Progress) -> Self {
        Self {
            requested_users: progress.total,
            task: progress.task,
        }
    }
}
