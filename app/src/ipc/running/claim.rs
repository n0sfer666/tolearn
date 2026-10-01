use std::thread;

use tolearn_provider::Stop;

use super::Running;
use super::finale::Finale;

#[derive(Debug)]
pub struct Claim {
    running: Running,
    stop: Stop,
    finale: Option<Finale>,
}

impl Claim {
    pub(super) fn new(running: Running, stop: Stop) -> Self {
        Self {
            running,
            stop,
            finale: None,
        }
    }

    pub fn stop(&self) -> &Stop {
        &self.stop
    }

    pub fn end(mut self, finale: Option<Finale>) {
        self.finale = finale;
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        let finale = self
            .finale
            .take()
            .or_else(|| thread::panicking().then_some(Finale::Crashed));
        self.running.release(finale);
    }
}
