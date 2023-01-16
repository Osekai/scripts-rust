use serde::Serialize;

use crate::{task::Task, util::Eta};

#[derive(Serialize)]
pub struct Progress {
    current: usize,
    total: usize,
    eta_seconds: Option<u64>,
    task: Task,
    users_per_sec: f32,
}

impl Progress {
    pub const INTERVAL: usize = 100;

    pub fn new(total: usize, task: Task) -> Self {
        Self {
            task,
            total,
            current: 0,
            eta_seconds: None,
            users_per_sec: 0.0,
        }
    }

    pub fn update(&mut self, current: usize, eta: &Eta) {
        self.current = current;

        let remaining = eta.estimate(self.total - current);
        self.eta_seconds = remaining.as_seconds();

        let elapsed = eta.get(Self::INTERVAL).elapsed();
        self.users_per_sec = (1000 * Self::INTERVAL) as f32 / elapsed.as_millis() as f32;
    }

    pub fn finish(&mut self, remaining: usize, eta: &Eta) {
        self.current = self.total;
        self.eta_seconds = Some(0);
        let elapsed = eta.get(remaining).elapsed();
        self.users_per_sec = (1000 * remaining) as f32 / elapsed.as_millis() as f32;
    }
}

#[derive(Copy, Clone, Serialize)]
pub struct Finish {
    pub requested_users: usize,
    pub task: Task,
}

impl From<Progress> for Finish {
    #[inline]
    fn from(progress: Progress) -> Self {
        Self {
            requested_users: progress.total,
            task: progress.task,
        }
    }
}
