use std::io::Write;
use std::path::Path;
use std::process::{ChildStdin, Command, Stdio};

use super::error::RunError;
use super::types::{Limits, Run};
use super::{drain, group, wait};

pub fn spawn(
    program: &str,
    args: &[String],
    directory: &Path,
    input: &str,
    limits: Limits,
) -> Result<Run, RunError> {
    if !directory.is_dir() {
        return Err(RunError::NoDirectory(directory.to_owned()));
    }
    let mut started = Command::new(program);
    started
        .args(args)
        .current_dir(directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = group::lead(&mut started)
        .spawn()
        .map_err(RunError::NotStarted)?;

    let out = drain::start(child.stdout.take(), limits.output_bytes);
    let err = drain::start(child.stderr.take(), limits.output_bytes);
    feed(child.stdin.take(), input);
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

fn feed(pipe: Option<ChildStdin>, input: &str) {
    let sent = input.as_bytes().to_vec();
    std::thread::spawn(move || {
        let Some(mut pipe) = pipe else {
            return;
        };
        let _ = pipe.write_all(&sent);
        let _ = pipe.flush();
    });
}
