#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

mod support;

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use support::harness;
use tolearn_provider::{Api, CheckError, Http, Kind, Provider, Stop, stoppable};

fn pressed_after(stop: &Stop, millis: u64) {
    let stop = stop.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(millis));
        stop.stop();
    });
}

fn pressed_once_written(stop: &Stop, pid: &Path) {
    let (stop, pid) = (stop.clone(), pid.to_path_buf());
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline && !written(&pid) {
            std::thread::sleep(Duration::from_millis(20));
        }
        stop.stop();
    });
}

fn written(pid: &Path) -> bool {
    std::fs::read_to_string(pid).is_ok_and(|text| !text.trim().is_empty())
}

fn ollama(endpoint: String) -> Provider {
    Provider {
        enabled: true,
        active: Kind::Local,
        local: Http {
            endpoint,
            api: Api::Ollama,
            model: "llama3:8b".to_owned(),
            ..Http::local()
        },
        ..Provider::default()
    }
}

fn pid_file() -> PathBuf {
    let pid = std::env::temp_dir().join(format!("tolearn-linger-{}.pid", std::process::id()));
    let _ = std::fs::remove_file(&pid);
    pid
}

#[test]
#[cfg(unix)]
fn a_cancelled_harness_leaves_no_process_behind() {
    let pid = pid_file();
    let provider = harness(
        &["linger".to_owned(), pid.to_string_lossy().into_owned()],
        60,
    );
    let stop = Stop::default();
    pressed_once_written(&stop, &pid);

    let started = Instant::now();
    let error = stoppable(&provider, None, "спроси", &stop).unwrap_err();

    assert_eq!(error, CheckError::Cancelled);
    assert_eq!(error.code(), "provider.cancelled");
    assert!(started.elapsed() < Duration::from_secs(30));
    let lingered = std::fs::read_to_string(&pid).unwrap();
    let alive = std::process::Command::new("kill")
        .args(["-0", lingered.trim()])
        .status()
        .unwrap();
    assert!(
        !alive.success(),
        "харнесс {} пережил отмену",
        lingered.trim()
    );
    let _ = std::fs::remove_file(&pid);
}

#[test]
fn a_cancelled_request_stops_waiting_for_a_silent_server() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let held: Vec<_> = listener.incoming().take(1).flatten().collect();
        std::thread::sleep(Duration::from_secs(30));
        drop(held);
    });
    let provider = ollama(endpoint);
    let stop = Stop::default();
    pressed_after(&stop, 300);

    let started = Instant::now();
    let error = stoppable(&provider, None, "спроси", &stop).unwrap_err();

    assert_eq!(error, CheckError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_stop_pressed_before_the_call_sends_nothing() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let provider = ollama(format!("http://{}", listener.local_addr().unwrap()));
    let stop = Stop::default();
    stop.stop();

    let error = stoppable(&provider, None, "спроси", &stop).unwrap_err();

    assert_eq!(error, CheckError::Cancelled);
    std::thread::sleep(Duration::from_millis(300));
    assert!(listener.accept().is_err(), "запрос ушёл после отмены");
}

#[test]
fn an_unpressed_stop_changes_nothing() {
    let answer = stoppable(
        &harness(&["say".to_owned()], 20),
        None,
        "привет",
        &Stop::default(),
    )
    .unwrap();

    assert_eq!(answer.text, "услышал: привет");
}
