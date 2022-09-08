use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    slice::Iter,
    str::FromStr,
};

use eyre::{Context as _, Report};

use crate::task::Task;

pub struct Schedule {
    tasks: Vec<Task>,
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
            .map(|task| {
                task.split('|')
                    .map(str::trim)
                    .map(Task::from_str)
                    .try_fold(Task::empty(), |total, next| next.map(|next| total | next))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self { tasks })
    }
}

impl Display for Schedule {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut iter = self.tasks.iter();

        if let Some(task) = iter.next() {
            write!(f, "{task}")?;

            for task in iter {
                write!(f, ", {task}")?;
            }

            Ok(())
        } else {
            f.write_str("No tasks")
        }
    }
}
