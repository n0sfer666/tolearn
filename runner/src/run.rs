use std::path::Path;
use std::process::{Command, Stdio};

use super::error::RunError;
use super::types::{Limits, Run};
use super::{drain, group, wait};

pub fn run(command: &str, directory: &Path, limits: Limits) -> Result<Run, RunError> {
    if !directory.is_dir() {
        return Err(RunError::NoDirectory(directory.to_owned()));
    }
    let mut started = Command::new("sh");
    started
        .arg("-c")
        .arg(command)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = group::lead(&mut started)
        .spawn()
        .map_err(RunError::NotStarted)?;

    let out = drain::start(child.stdout.take(), limits.output_bytes, None);
    let err = drain::start(child.stderr.take(), limits.output_bytes, None);
    let outcome = wait::until_end(&mut child, limits.timeout)?;
    let (stdout, cut_out) = drain::done(out);
    let (stderr, cut_err) = drain::done(err);

    Ok(Run {
        outcome,
        stdout,
        stderr,
        truncated: cut_out || cut_err,
    })
}
