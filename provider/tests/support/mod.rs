#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]
#![allow(dead_code, reason = "each test binary takes its own share of helpers")]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex, PoisonError};

use tolearn_provider::{Harness, Kind, Provider};

pub fn harness(args: &[String], timeout_secs: u32) -> Provider {
    Provider {
        enabled: true,
        active: Kind::Harness,
        harness: Harness {
            id: "custom".to_owned(),
            command: env!("CARGO_BIN_EXE_fake-harness").to_owned(),
            args: args.to_vec(),
            timeout_secs,
        },
        ..Provider::default()
    }
}

#[derive(Debug)]
pub struct Stub {
    pub endpoint: String,
    heard: Arc<Mutex<Vec<String>>>,
}

impl Stub {
    pub fn heard(&self) -> Vec<String> {
        self.heard
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

pub fn stub(status: &'static str, body: &'static str) -> Stub {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let heard = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&heard);

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(&stream);
            let mut head = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
                head.push_str(&line);
            }
            let mut sent = vec![0; length(&head)];
            let _ = reader.read_exact(&mut sent);
            head.push_str(&String::from_utf8_lossy(&sent));
            seen.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(head);

            let mut answer = &stream;
            let _ = write!(
                answer,
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = answer.flush();
        }
    });

    Stub { endpoint, heard }
}

fn length(head: &str) -> usize {
    head.lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())?
        })
        .unwrap_or(0)
}

pub fn closed() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    endpoint
}
