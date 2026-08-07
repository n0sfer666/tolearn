use std::process::Child;
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::beat::Beat;
use super::error::RunError;
use super::kill;
use super::types::{Limits, Outcome};

const POLL: Duration = Duration::from_millis(10);

pub fn until_end(child: &mut Child, limits: Limits, beat: &Beat) -> Result<Outcome, RunError> {
    let deadline = Instant::now() + limits.timeout;
    loop {
        match child.try_wait().map_err(RunError::Broken)? {
            Some(status) => {
                return Ok(Outcome::Finished {
                    code: status.code(),
                });
            }
            None if hushed(limits.silence, beat) => return cut(child, Outcome::WentQuiet),
            None if Instant::now() >= deadline => return cut(child, Outcome::TimedOut),
            None => sleep(POLL),
        }
    }
}

fn cut(child: &mut Child, outcome: Outcome) -> Result<Outcome, RunError> {
    kill::tree(child.id());
    child.wait().map_err(RunError::Broken)?;
    Ok(outcome)
}

fn hushed(silence: Option<Duration>, beat: &Beat) -> bool {
    match (silence, beat.quiet()) {
        (Some(allowed), Some(quiet)) => quiet >= allowed,
        _ => false,
    }
}
