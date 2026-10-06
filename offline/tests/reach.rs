#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use tolearn_offline::reach::{Ping, Reach};

fn local() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    (listener, url)
}

#[test]
fn any_answer_means_the_host_is_reachable() {
    let (listener, url) = local();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0; 1024];
        let _ = stream.read(&mut request);
        stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
    });

    assert_eq!(Ping::new(5).unwrap().reach(&url), Ok(()));
    server.join().unwrap();
}

#[test]
fn a_closed_port_gives_a_reason() {
    let (listener, url) = local();
    drop(listener);

    let reason = Ping::new(5).unwrap().reach(&url).unwrap_err();

    assert!(!reason.is_empty());
    assert!(!reason.contains("error sending request"), "{reason}");
}

#[test]
fn a_silent_host_fails_within_the_timeout() {
    let (listener, url) = local();
    let started = Instant::now();

    let reason = Ping::new(1).unwrap().reach(&url).unwrap_err();

    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
    assert!(reason.contains("timed out"), "{reason}");
    drop(listener);
}
