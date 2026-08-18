use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const NEVER: u64 = u64::MAX;

#[derive(Debug)]
pub struct Beat {
    since: Instant,
    last: AtomicU64,
}

impl Default for Beat {
    fn default() -> Self {
        Self {
            since: Instant::now(),
            last: AtomicU64::new(NEVER),
        }
    }
}

impl Beat {
    pub fn hit(&self) {
        let millis = u64::try_from(self.since.elapsed().as_millis()).unwrap_or(NEVER - 1);
        self.last.store(millis, Ordering::Relaxed);
    }

    pub fn quiet(&self) -> Option<Duration> {
        let last = self.last.load(Ordering::Relaxed);
        (last != NEVER).then(|| {
            self.since
                .elapsed()
                .saturating_sub(Duration::from_millis(last))
        })
    }
}
