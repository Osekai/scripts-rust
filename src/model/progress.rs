use serde::Serialize;

use crate::util::TimeEstimate;

#[derive(Serialize)]
pub struct Progress {
    current: usize,
    total: usize,
    eta_seconds: Option<u64>,
}

impl Progress {
    pub fn new(current: usize, total: usize, remaining: TimeEstimate) -> Self {
        Self {
            current,
            total,
            eta_seconds: remaining.as_seconds(),
        }
    }
}
