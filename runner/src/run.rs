use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::drain;
use super::error::RunError;
use super::types::{Limits, Outcome, Run};

const POLL: Duration = Duration::from_millis(10);

pub fn run(command: &str, directory: &Path, limits: Limits) -> Result<Run, RunError> {
    if !directory.is_dir() {
        return Err(RunError::NoDirectory(directory.to_owned()));
    }
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(RunError::NotStarted)?;

    let out = drain::start(child.stdout.take(), limits.output_bytes);
    let err = drain::start(child.stderr.take(), limits.output_bytes);
    let outcome = await_end(&mut child, limits.timeout)?;
    let (stdout, cut_out) = drain::done(out);
    let (stderr, cut_err) = drain::done(err);

    Ok(Run {
        outcome,
        stdout,
        stderr,
        truncated: cut_out || cut_err,
    })
}

fn await_end(child: &mut Child, timeout: Duration) -> Result<Outcome, RunError> {
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
    kill_tree(child.id());
    child.wait().map_err(RunError::Broken)?;
    Ok(Outcome::TimedOut)
}

fn kill_tree(group: u32) {
    let _ = Command::new("kill")
        .args(["-9", &format!("-{group}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
