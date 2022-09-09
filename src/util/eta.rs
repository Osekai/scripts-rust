use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    time::{Duration, Instant},
};

const BACKLOG_LEN: usize = 200;

/// Limited capacity queue of time instances.
pub struct Eta {
    queue: Vec<Instant>,
    /// If the queue is not empty, `end` is the index of the last element.
    /// Otherwise, it has no meaning.
    end: usize,
    /// Amount of elements in the queue. This is equal to `end + 1`
    /// if the queue is not full, or `BACKLOG_LEN` otherwise.
    len: usize,
}

impl Eta {
    pub fn tick(&mut self) {
        self.end = (self.end + 1) % BACKLOG_LEN;
        self.queue[self.end] = Instant::now();
        self.len += (self.len < BACKLOG_LEN) as usize;
    }

    pub fn estimate(&self, remaining: usize) -> TimeEstimate {
        TimeEstimate(self.estimate_(remaining))
    }

    fn estimate_(&self, remaining: usize) -> Option<Duration> {
        let last = *self.queue.get(self.end).filter(|_| self.len > 20)?;

        let first_idx = ((self.len == BACKLOG_LEN) as usize * (self.end + 1)) % BACKLOG_LEN;
        let first = self.queue[first_idx];

        let remaining_for_one = (last - first).as_millis() as f64 / self.len as f64;
        let eta_millis = remaining_for_one * remaining as f64;

        Some(Duration::from_millis(eta_millis as u64))
    }
}

impl Default for Eta {
    #[inline]
    fn default() -> Self {
        Self {
            end: BACKLOG_LEN - 1,
            queue: vec![Instant::now(); BACKLOG_LEN],
            len: 0,
        }
    }
}

pub struct TimeEstimate(Option<Duration>);

impl Display for TimeEstimate {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(duration) = self.0 {
            let mut secs = duration.as_secs();

            let hours = secs / 3600;
            secs %= 3600;

            let minutes = secs / 60;
            secs %= 60;

            f.write_str("~")?;

            if hours > 0 {
                write!(f, "{hours}h{minutes}m")?;
            } else if minutes > 0 {
                write!(f, "{minutes}m")?;
            }

            write!(f, "{secs}s")
        } else {
            f.write_str("N/A")
        }
    }
}
