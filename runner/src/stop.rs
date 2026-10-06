use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

const GOING: u8 = 0;
const STOPPED: u8 = 1;
const SEALED: u8 = 2;

#[derive(Debug, Clone, Default)]
pub struct Stop(Arc<AtomicU8>);

impl Stop {
    pub fn stop(&self) -> bool {
        self.turn(STOPPED)
    }

    pub fn seal(&self) -> bool {
        self.turn(SEALED)
    }

    pub fn stopped(&self) -> bool {
        self.0.load(Ordering::SeqCst) == STOPPED
    }

    fn turn(&self, to: u8) -> bool {
        match self
            .0
            .compare_exchange(GOING, to, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(_) => true,
            Err(now) => now == to,
        }
    }
}
