#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]
#![allow(dead_code, reason = "опоры нужны не каждому тест-бинарнику")]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

static COPIES: AtomicUsize = AtomicUsize::new(0);

pub struct Stub {
    pub endpoint: String,
    heard: Arc<Mutex<Vec<String>>>,
}

impl Stub {
    #[allow(dead_code, reason = "нужна не каждому тест-бинарнику")]
    pub fn heard(&self) -> Vec<String> {
        self.heard
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[allow(dead_code, reason = "нужна не каждому тест-бинарнику")]
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

pub fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("рядом с app лежит корень репозитория")
        .to_path_buf()
}

pub fn copied(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "tolearn-app-{name}-{}-{}",
        std::process::id(),
        COPIES.fetch_add(1, Ordering::Relaxed)
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    copy(&repository().join("examples/llm-agents-base"), &directory);
    strip(&directory, "json");
    directory
}

fn strip(directory: &Path, extension: &str) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            strip(&path, extension);
            continue;
        }
        if path.extension().is_some_and(|found| found == extension) {
            std::fs::remove_file(path).unwrap();
        }
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[allow(dead_code, reason = "нужна не каждому тест-бинарнику")]
pub fn sources(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
            continue;
        }
        if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}
