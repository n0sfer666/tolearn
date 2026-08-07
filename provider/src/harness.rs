use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tolearn_runner::{Limits, Outcome, RunError, Seen, spawn};

use crate::ansi::plain;
use crate::ask::Said;
use crate::error::CheckError;
use crate::scratch::Scratch;
use crate::stream::Tape;
use crate::types::{Harness, Watch};

pub const OUTPUT_BYTES: usize = 1024 * 1024;
pub const VERSION_TIMEOUT: Duration = Duration::from_secs(20);

const COMPLAINT_LINES: usize = 3;
const COMPLAINT_CHARS: usize = 300;

pub fn version(harness: &Harness) -> Result<String, CheckError> {
    let (said, complained) = ran(
        harness,
        &["--version".to_owned()],
        "",
        VERSION_TIMEOUT,
        None,
    )?;
    Ok(first(&said).unwrap_or_else(|| first(&complained).unwrap_or_default()))
}

pub fn ask(harness: &Harness, prompt: &str, watch: Option<Watch>) -> Result<Said, CheckError> {
    let patience = Duration::from_secs(u64::from(harness.timeout_secs));
    let tape = Arc::new(Mutex::new(Tape::default()));
    let (said, _) = ran(
        harness,
        &harness.args,
        prompt,
        patience,
        Some(watcher(&tape, watch)),
    )?;
    let said = tape
        .lock()
        .ok()
        .and_then(|tape| tape.heard())
        .unwrap_or_else(|| plainly(said));
    match said.text.is_empty() {
        true => Err(CheckError::BadAnswer),
        false => Ok(said),
    }
}

fn watcher(tape: &Arc<Mutex<Tape>>, watch: Option<Watch>) -> Seen {
    let tape = Arc::clone(tape);
    Arc::new(move |chunk: &str| {
        let shown = tape
            .lock()
            .map(|mut tape| tape.feed(&plain(chunk)))
            .unwrap_or_default();
        match watch.as_ref() {
            Some(watch) if !shown.is_empty() => watch(&shown),
            _ => {}
        }
    })
}

fn plainly(text: String) -> Said {
    Said {
        text: text.trim().to_owned(),
        thinking: false,
        tokens: None,
    }
}

fn ran(
    harness: &Harness,
    args: &[String],
    input: &str,
    timeout: Duration,
    seen: Option<Seen>,
) -> Result<(String, String), CheckError> {
    let command = harness.command.trim();
    if command.is_empty() {
        return Err(CheckError::NotFound(String::new()));
    }
    let scratch = Scratch::new().map_err(|error| CheckError::Unreachable(error.to_string()))?;
    let limits = Limits {
        timeout,
        output_bytes: OUTPUT_BYTES,
    };

    let run = spawn(command, args, scratch.path(), input, limits, seen)
        .map_err(|error| broken(command, error))?;
    match run.outcome {
        Outcome::TimedOut => Err(CheckError::TimedOut(seconds(timeout))),
        Outcome::Finished { code } if code != Some(0) => Err(CheckError::Failed {
            code,
            said: complaint(&run.stderr),
        }),
        Outcome::Finished { .. } if run.truncated => Err(CheckError::Truncated),
        Outcome::Finished { .. } => Ok((plain(&run.stdout), plain(&run.stderr))),
    }
}

fn broken(command: &str, error: RunError) -> CheckError {
    match error {
        RunError::NotStarted(started) if started.kind() == ErrorKind::NotFound => {
            CheckError::NotFound(command.to_owned())
        }
        other => CheckError::Unreachable(other.to_string()),
    }
}

fn complaint(stderr: &str) -> String {
    plain(stderr)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(COMPLAINT_LINES)
        .collect::<Vec<_>>()
        .join("; ")
        .chars()
        .take(COMPLAINT_CHARS)
        .collect()
}

fn first(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

fn seconds(timeout: Duration) -> u32 {
    u32::try_from(timeout.as_secs()).unwrap_or(u32::MAX)
}
