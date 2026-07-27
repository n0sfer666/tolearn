#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "runner gate: a panic here is the report"
)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use tolearn_runner::{Limits, Outcome, run};

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-runner-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn limits(millis: u64) -> Limits {
    Limits {
        timeout: Duration::from_millis(millis),
        output_bytes: 64 * 1024,
    }
}

#[test]
fn a_command_that_succeeds_reports_its_code_and_what_it_printed() {
    let directory = scratch("ok");

    let run = run("echo READY", &directory, limits(5_000)).unwrap();

    assert_eq!(run.outcome, Outcome::Finished { code: Some(0) });
    assert_eq!(run.stdout.trim(), "READY");
    assert!(!run.truncated);
}

#[test]
fn a_command_that_fails_keeps_its_code_and_its_complaint() {
    let directory = scratch("fail");

    let run = run("echo BROKEN >&2; exit 3", &directory, limits(5_000)).unwrap();

    assert_eq!(run.outcome, Outcome::Finished { code: Some(3) });
    assert_eq!(run.stderr.trim(), "BROKEN");
    assert!(run.stdout.is_empty());
}

#[test]
fn a_command_runs_in_the_directory_it_was_given() {
    let directory = scratch("cwd");
    std::fs::write(directory.join("artifact.md"), "тут").unwrap();

    let run = run(
        "test -s artifact.md && echo HERE",
        &directory,
        limits(5_000),
    )
    .unwrap();

    assert_eq!(run.stdout.trim(), "HERE");
}

#[test]
fn a_command_that_does_not_stop_is_cut_off_by_the_timeout() {
    let directory = scratch("timeout");
    let started = Instant::now();

    let run = run("sleep 30", &directory, limits(400)).unwrap();

    assert_eq!(run.outcome, Outcome::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn the_children_of_a_command_die_with_it() {
    let directory = scratch("tree");
    let pid = directory.join("child.pid");
    let command = format!(
        "sh -c 'sleep 30 & echo $! > {}; wait'",
        pid.to_string_lossy()
    );

    let started = Instant::now();
    let run = run(&command, &directory, limits(600)).unwrap();

    assert_eq!(run.outcome, Outcome::TimedOut);
    assert!(
        started.elapsed() < Duration::from_secs(5),
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

#[test]
fn a_talkative_command_is_cut_at_the_limit_and_says_so() {
    let directory = scratch("flood");
    let limits = Limits {
        timeout: Duration::from_secs(10),
        output_bytes: 2_000,
    };

    let run = run("seq 1 200000", &directory, limits).unwrap();

    assert!(run.truncated);
    assert!(run.stdout.len() <= 2_000);
}

#[test]
fn a_command_that_waits_for_input_gets_none_and_does_not_hang() {
    let directory = scratch("stdin");
    let started = Instant::now();

    let run = run(
        "read answer; echo GOT ${answer:-NOTHING}",
        &directory,
        limits(5_000),
    )
    .unwrap();

    assert_eq!(run.stdout.trim(), "GOT NOTHING");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_directory_that_is_not_there_is_an_error_not_a_run() {
    let directory = scratch("missing").join("gone");

    let error = run("echo READY", &directory, limits(5_000)).unwrap_err();

    assert!(!error.to_string().is_empty());
}

#[test]
fn a_command_of_the_reference_bundle_runs_as_it_is_written() {
    let directory = scratch("bundle");
    std::fs::write(directory.join("Modelfile"), "PARAMETER num_ctx 32768\n").unwrap();
    let command = "bash -c 'test -s Modelfile && grep -Eiq \"^[[:space:]]*PARAMETER[[:space:]]+num_ctx[[:space:]]+32768\" Modelfile && echo MODELFILE_OK || echo MODELFILE_FAIL'";

    let run = run(command, &directory, limits(5_000)).unwrap();

    assert_eq!(run.stdout.trim(), "MODELFILE_OK");
}

#[test]
fn a_command_killed_by_a_signal_is_finished_without_a_code() {
    let directory = scratch("signal");

    let run = run("kill -9 $$", &directory, limits(5_000)).unwrap();

    assert_eq!(run.outcome, Outcome::Finished { code: None });
}
