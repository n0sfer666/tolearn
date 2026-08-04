use std::process::Child;
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::error::RunError;
use super::kill;
use super::types::Outcome;

const POLL: Duration = Duration::from_millis(10);

pub fn until_end(child: &mut Child, timeout: Duration) -> Result<Outcome, RunError> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait().map_err(RunError::Broken)? {
            Some(status) => {
                return Ok(Outcome::Finished {
                    code: status.code(),
                });
            }
            None if Instant::now() >= deadline => break,
            None => sleep(POLL),
        }
    }
    kill::tree(child.id());
    child.wait().map_err(RunError::Broken)?;
    Ok(Outcome::TimedOut)
}
