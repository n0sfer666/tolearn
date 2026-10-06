#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "runner gate: a panic here is the report"
)]

mod support;

use std::time::{Duration, Instant};

use support::{SHELL, SLEEP, args, limits, scratch};
use tolearn_runner::{Outcome, Stop, spawn};

#[test]
#[cfg(unix)]
fn a_stopped_program_dies_with_its_children_at_once() {
    let directory = scratch("stop");
    let pid = directory.join("child.pid");
    let script = format!(
        "sh -c 'sleep 30 & echo $! > {}; wait'",
        pid.to_string_lossy()
    );
    let stop = Stop::default();
    let pressed = stop.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(400));
        pressed.stop();
    });

    let started = Instant::now();
    let run = spawn(
        SHELL.0,
        &args(&script),
        &directory,
        "",
        limits(20_000, 64 * 1024),
        None,
        &stop,
    )
    .unwrap();

    assert_eq!(run.outcome, Outcome::Stopped);
    assert!(started.elapsed() < Duration::from_secs(5));
    std::thread::sleep(Duration::from_millis(300));
    let child = std::fs::read_to_string(&pid).unwrap();
    let alive = std::process::Command::new("kill")
        .args(["-0", child.trim()])
        .status()
        .unwrap();
    assert!(!alive.success(), "процесс {} пережил отмену", child.trim());
}

#[test]
fn a_program_stopped_before_it_starts_is_cut_off_without_waiting() {
    let directory = scratch("stopped");
    let stop = Stop::default();
    stop.stop();

    let started = Instant::now();
    let run = spawn(
        SHELL.0,
        &args(SLEEP),
        &directory,
        "",
        limits(20_000, 64 * 1024),
        None,
        &stop,
    )
    .unwrap();

    assert_eq!(run.outcome, Outcome::Stopped);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_sealed_stop_refuses_to_stop_and_a_stopped_one_refuses_to_seal() {
    let sealed = Stop::default();
    assert!(sealed.seal());
    assert!(sealed.seal());
    assert!(!sealed.stop());
    assert!(!sealed.stopped());

    let stopped = Stop::default();
    assert!(stopped.stop());
    assert!(stopped.stop());
    assert!(!stopped.seal());
    assert!(stopped.stopped());
}
