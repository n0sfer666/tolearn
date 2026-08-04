#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "runner gate: a panic here is the report"
)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use tolearn_runner::{Limits, Outcome, spawn};

#[cfg(unix)]
const SHELL: (&str, &str) = ("sh", "-c");
#[cfg(windows)]
const SHELL: (&str, &str) = ("cmd", "/C");

#[cfg(unix)]
const ECHO_STDIN: &str = "cat";
#[cfg(windows)]
const ECHO_STDIN: &str = "findstr \"^\"";

#[cfg(unix)]
const SLEEP: &str = "sleep 30";
#[cfg(windows)]
const SLEEP: &str = "ping -n 31 127.0.0.1 >nul";

#[cfg(unix)]
const FLOOD: &str = "seq 1 200000";
#[cfg(windows)]
const FLOOD: &str = "for /L %i in (1,1,200000) do @echo %i";

#[cfg(unix)]
const FAIL: &str = "echo BROKEN >&2; exit 3";
#[cfg(windows)]
const FAIL: &str = "echo BROKEN 1>&2& exit /B 3";

#[cfg(unix)]
const LIST: &str = "ls";
#[cfg(windows)]
const LIST: &str = "dir /B";

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-spawn-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn args(script: &str) -> Vec<String> {
    vec![SHELL.1.to_owned(), script.to_owned()]
}

fn limits(millis: u64, bytes: usize) -> Limits {
    Limits {
        timeout: Duration::from_millis(millis),
        output_bytes: bytes,
    }
}

#[test]
fn the_prompt_reaches_the_program_through_its_stdin() {
    let directory = scratch("stdin");

    let run = spawn(
        SHELL.0,
        &args(ECHO_STDIN),
        &directory,
        "ГОТОВ",
        limits(10_000, 64 * 1024),
    )
    .unwrap();

    assert_eq!(run.outcome, Outcome::Finished { code: Some(0) });
    assert_eq!(run.stdout.trim(), "ГОТОВ");
}

#[test]
fn a_program_sees_only_the_directory_it_was_given() {
    let directory = scratch("cwd");

    let run = spawn(
        SHELL.0,
        &args(LIST),
        &directory,
        "",
        limits(10_000, 64 * 1024),
    )
    .unwrap();

    assert_eq!(run.outcome, Outcome::Finished { code: Some(0) });
    assert!(run.stdout.trim().is_empty(), "{}", run.stdout);
}

#[test]
fn a_program_that_does_not_stop_is_cut_off_by_the_timeout() {
    let directory = scratch("timeout");
    let started = Instant::now();

    let run = spawn(
        SHELL.0,
        &args(SLEEP),
        &directory,
        "",
        limits(600, 64 * 1024),
    )
    .unwrap();

    assert_eq!(run.outcome, Outcome::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn a_talkative_program_is_cut_at_the_limit_and_says_so() {
    let directory = scratch("flood");

    let run = spawn(SHELL.0, &args(FLOOD), &directory, "", limits(20_000, 2_000)).unwrap();

    assert!(run.truncated);
    assert!(run.stdout.len() <= 2_000);
}

#[test]
fn a_program_that_fails_keeps_its_code_and_its_complaint() {
    let directory = scratch("fail");

    let run = spawn(
        SHELL.0,
        &args(FAIL),
        &directory,
        "",
        limits(10_000, 64 * 1024),
    )
    .unwrap();

    assert_eq!(run.outcome, Outcome::Finished { code: Some(3) });
    assert_eq!(run.stderr.trim(), "BROKEN");
}

#[test]
fn a_program_that_is_not_installed_is_an_error_not_a_run() {
    let directory = scratch("missing");

    let error = spawn(
        "tolearn-no-such-program",
        &[],
        &directory,
        "",
        limits(10_000, 64 * 1024),
    )
    .unwrap_err();

    assert!(!error.to_string().is_empty());
}

#[test]
fn a_directory_that_is_not_there_is_an_error_not_a_run() {
    let directory = scratch("gone").join("nowhere");

    let error = spawn(
        SHELL.0,
        &args(LIST),
        &directory,
        "",
        limits(10_000, 64 * 1024),
    )
    .unwrap_err();

    assert!(!error.to_string().is_empty());
}

#[test]
#[cfg(unix)]
fn the_children_of_a_program_die_with_it() {
    let directory = scratch("tree");
    let pid = directory.join("child.pid");
    let script = format!(
        "sh -c 'sleep 30 & echo $! > {}; wait'",
        pid.to_string_lossy()
    );

    let started = Instant::now();
    let run = spawn(
        SHELL.0,
        &args(&script),
        &directory,
        "",
        limits(800, 64 * 1024),
    )
    .unwrap();

    assert_eq!(run.outcome, Outcome::TimedOut);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "раннер ждал потомков вместо того, чтобы убить их"
    );
    std::thread::sleep(Duration::from_millis(300));
    let child = std::fs::read_to_string(&pid).unwrap();
    let alive = std::process::Command::new("kill")
        .args(["-0", child.trim()])
        .status()
        .unwrap();
    assert!(!alive.success(), "процесс {} пережил таймаут", child.trim());
}
