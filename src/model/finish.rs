use serde::Serialize;

use crate::task::Task;

#[derive(Copy, Clone, Serialize)]
pub struct Finish {
    pub requested_users: u32,
    pub task: Task,
}

impl Finish {
    pub fn new(task: Task, requested_users: u32) -> Self {
        Self {
            task,
            requested_users,
        }
    }
}
