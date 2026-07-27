use std::time::Duration;

use tolearn_core::topic::Check;
use tolearn_runner::{Limits, Outcome, run as execute};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{RunCheckIn, RunCheckOut};

const LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    output_bytes: 64 * 1024,
};

pub fn run(_context: &Context, input: &RunCheckIn) -> Result<RunCheckOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let topic = opened
        .scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;
    let check = topic
        .practice
        .constraints
        .iter()
        .chain(topic.practice.acceptance.iter())
        .find(|check| check.id == input.check)
        .ok_or_else(|| unknown_check(&input.check))?;

    finished(check, &opened.scan.root)
}

fn finished(check: &Check, root: &std::path::Path) -> Result<RunCheckOut, IpcError> {
    let done = execute(&check.check, root, LIMITS).map_err(|error| {
        IpcError::new(
            "check.unrunnable",
            format!("проверка `{}` не запускается: {error}", check.id),
        )
    })?;

    Ok(RunCheckOut {
        id: check.id.clone(),
        command: check.check.clone(),
        expect: check.expect.clone(),
        code: code_of(done.outcome),
        timed_out: done.outcome == Outcome::TimedOut,
        stdout: done.stdout,
        stderr: done.stderr,
        truncated: done.truncated,
    })
}

fn code_of(outcome: Outcome) -> Option<i32> {
    match outcome {
        Outcome::Finished { code } => code,
        Outcome::TimedOut => None,
    }
}

fn unknown_check(id: &str) -> IpcError {
    IpcError::new("check.unknown", format!("тема не знает проверку `{id}`"))
}

#[cfg(test)]
mod tests {
    use super::{Outcome, code_of};

    #[test]
    fn истёкшее_время_не_выдаёт_себя_за_нулевой_код() {
        assert_eq!(code_of(Outcome::TimedOut), None);
        assert_eq!(code_of(Outcome::Finished { code: Some(0) }), Some(0));
        assert_eq!(code_of(Outcome::Finished { code: None }), None);
    }
}
