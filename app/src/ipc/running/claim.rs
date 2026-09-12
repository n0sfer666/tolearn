use tolearn_provider::Stop;

use super::Running;

#[derive(Debug)]
pub struct Claim {
    running: Running,
    stop: Stop,
}

impl Claim {
    pub(super) fn new(running: Running, stop: Stop) -> Self {
        Self { running, stop }
    }

    pub fn stop(&self) -> &Stop {
        &self.stop
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        self.running.release();
    }
}
