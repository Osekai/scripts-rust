use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    slice::Iter,
    str::FromStr,
};

use eyre::Report;

use crate::task::Task;

/// Contains a list of tasks to be executed one after
/// the other with an interval in between.
pub struct Schedule {
    tasks: Box<[Task]>,
}

impl Schedule {
    pub fn iter(&self) -> Iter<'_, Task> {
        self.tasks.iter()
    }
}

impl FromStr for Schedule {
    type Err = Report;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tasks = s
            .split(',')
            .map(str::trim)
            .map(Task::from_str)
            .collect::<Result<_, _>>()
            .map(Vec::into_boxed_slice)?;

        Ok(Self { tasks })
    }
}

impl Display for Schedule {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut iter = self.tasks.iter();

        if let Some(task) = iter.next() {
            Display::fmt(task, f)?;

            for task in iter {
                write!(f, ", {task}")?;
            }

            Ok(())
        } else {
            f.write_str("No tasks")
        }
    }
}
