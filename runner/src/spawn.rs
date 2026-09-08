use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;

use super::beat::Beat;
use super::error::RunError;
use super::types::{Limits, Run, Seen};
use super::{drain, group, path, wait};

pub fn spawn(
    program: &str,
    args: &[String],
    directory: &Path,
    input: &str,
    limits: Limits,
    seen: Option<Seen>,
) -> Result<Run, RunError> {
    if !directory.is_dir() {
        return Err(RunError::NoDirectory(directory.to_owned()));
    }
    let mut child = launched(program, args, directory)?;

    let beat = Arc::new(Beat::default());
    let out = drain::start(
        child.stdout.take(),
        limits.output_bytes,
        seen,
        Arc::clone(&beat),
    );
    let err = drain::start(
        child.stderr.take(),
        limits.output_bytes,
        None,
        Arc::clone(&beat),
    );
    feed(child.stdin.take(), input);
    let outcome = wait::until_end(&mut child, limits, &beat)?;
    let (stdout, cut_out) = drain::done(out);
    let (stderr, cut_err) = drain::done(err);

    Ok(Run {
        outcome,
        stdout,
        stderr,
        truncated: cut_out || cut_err,
    })
}

fn launched(program: &str, args: &[String], directory: &Path) -> Result<Child, RunError> {
    let search = path::search();
    let mut refused = None;
    for found in path::candidates(program, &search) {
        let mut started = Command::new(found);
        started
            .args(args)
            .env("PATH", &search)
            .current_dir(directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match group::lead(&mut started).spawn() {
            Ok(child) => return Ok(child),
            Err(error) => {
                refused.get_or_insert(error);
            }
        }
    }
    Err(RunError::NotStarted(refused.unwrap_or_else(|| {
        std::io::Error::from(ErrorKind::NotFound)
    })))
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
