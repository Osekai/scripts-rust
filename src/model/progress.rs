use serde::Serialize;

use crate::{task::Task, util::TimeEstimate};

#[derive(Serialize)]
pub struct Progress {
    current: u32,
    total: u32,
    eta_seconds: Option<u64>,
    task: Task,
    users_per_sec: f32,
}

impl Progress {
    pub fn new(total: u32, task: Task) -> Self {
        Self {
            task,
            total,
            current: 0,
            eta_seconds: None,
            users_per_sec: 0.0,
        }
    }

    pub fn update(&mut self, current: u32, remaining: TimeEstimate, users_per_sec: f32) {
        self.current = current;
        self.eta_seconds = remaining.as_seconds();
        self.users_per_sec = users_per_sec;
    }
}
