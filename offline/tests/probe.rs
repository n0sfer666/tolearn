#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex, PoisonError};

use tolearn_offline::fresh::{Answer, Conditional, Probe};

struct Server {
    address: String,
    heard: Arc<Mutex<Vec<String>>>,
}

impl Server {
    fn heard(&self) -> Vec<String> {
        self.heard
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

fn serving(answers: Vec<String>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let heard = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&heard);

    std::thread::spawn(move || {
        for (stream, answer) in listener.incoming().flatten().zip(answers) {
            let mut reader = BufReader::new(&stream);
            let mut head = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
                head.push_str(&line);
            }
            seen.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(head);
            let mut writer = &stream;
            let _ = writer.write_all(answer.as_bytes());
            let _ = writer.flush();
        }
    });

    Server { address, heard }
}

fn sent(status: &str, headers: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\n{headers}content-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[test]
fn ответ_304_доказывает_что_ничего_не_поменялось() {
    let server = serving(vec![
        "HTTP/1.1 304 Not Modified\r\nconnection: close\r\n\r\n".to_string(),
    ]);
    let probe = Conditional::new(5).unwrap();

    let answer = probe
        .ask(
            &server.address,
            Some("\"v1\""),
            Some("Mon, 27 Jul 2026 10:00:00 GMT"),
        )
        .unwrap();

    assert_eq!(answer, Answer::Same);
    let head = server.heard().first().cloned().unwrap_or_default();
    assert!(head.contains("if-none-match: \"v1\""), "{head}");
    assert!(
        head.contains("if-modified-since: Mon, 27 Jul 2026 10:00:00 GMT"),
        "{head}"
    );
}

#[test]
fn ответ_200_приносит_тело_и_новые_валидаторы() {
    let server = serving(vec![sent(
        "200 OK",
        "etag: \"v2\"\r\nlast-modified: Tue, 28 Jul 2026 10:00:00 GMT\r\n",
        "<html><body>тело</body></html>",
    )]);
    let probe = Conditional::new(5).unwrap();

    let answer = probe.ask(&server.address, None, None).unwrap();

    assert_eq!(
        answer,
        Answer::Sent {
            bytes: b"<html><body>\xd1\x82\xd0\xb5\xd0\xbb\xd0\xbe</body></html>".to_vec(),
            etag: Some("\"v2\"".to_string()),
            last_modified: Some("Tue, 28 Jul 2026 10:00:00 GMT".to_string()),
        }
    );
}

#[test]
fn без_валидаторов_условных_заголовков_не_шлётся() {
    let server = serving(vec![sent("200 OK", "", "тело")]);
    let probe = Conditional::new(5).unwrap();

    probe.ask(&server.address, None, None).unwrap();

    let head = server.heard().first().cloned().unwrap_or_default();
    assert!(!head.contains("if-none-match"), "{head}");
    assert!(!head.contains("if-modified-since"), "{head}");
}

#[test]
fn тело_больше_потолка_хэшем_не_прикидывается() {
    let huge = "я".repeat(3 * 1024 * 1024);
    let server = serving(vec![sent("200 OK", "etag: \"v9\"\r\n", &huge)]);
    let probe = Conditional::new(5).unwrap();

    let answer = probe.ask(&server.address, None, None).unwrap();

    assert_eq!(
        answer,
        Answer::Big {
            etag: Some("\"v9\"".to_string()),
            last_modified: None,
        },
        "обрезок ушёл в хэш и притворился телом"
    );
}

#[test]
fn чужая_ошибка_остаётся_ошибкой_а_не_свежестью() {
    let server = serving(vec![sent("404 Not Found", "", "нет такого")]);
    let probe = Conditional::new(5).unwrap();

    let answer = probe.ask(&server.address, None, None);

    assert!(answer.is_err(), "{answer:?}");
    assert!(answer.unwrap_err().contains("404"));
}
