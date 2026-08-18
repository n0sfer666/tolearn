#![allow(
    clippy::expect_used,
    reason = "app example: a panic here is the report"
)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

use serde_json::{Value, json};
use tolearn_app::ipc::{Context, call, layout};

const ADDRESS: &str = "127.0.0.1:4319";
const LIMIT: usize = 8 * 1024 * 1024;

fn main() {
    let home = PathBuf::from(std::env::var("HOME").expect("нет HOME"));
    let places = layout::places(&home);
    for room in [&places.config, &places.data] {
        std::fs::create_dir_all(room).expect("каталог не создан");
    }
    let context = Context::split(&places.config, &places.data);
    let listener = TcpListener::bind(ADDRESS).expect("порт занят");
    println!("мост: http://{ADDRESS}");
    println!("конфиг: {}", places.config.display());
    println!("данные: {}", places.data.display());
    for stream in listener.incoming().flatten() {
        let mine = context.clone();
        std::thread::spawn(move || serve(stream, &mine));
    }
}

fn serve(mut stream: TcpStream, context: &Context) {
    let Some(asked) = taken(&mut stream) else {
        return;
    };
    if asked.is_null() {
        answer(&mut stream, 204, "");
        return;
    }
    let name = asked["name"].as_str().unwrap_or_default().to_owned();
    let payload = asked.get("payload").cloned().unwrap_or_else(|| json!({}));
    match call(context, &name, &payload) {
        Ok(out) => {
            println!("ок  {name}");
            answer(&mut stream, 200, &out.to_string());
        }
        Err(error) => {
            let body = serde_json::to_string(&error).unwrap_or_default();
            println!("НЕТ {name}: {body}");
            answer(&mut stream, 500, &body);
        }
    }
}

fn taken(stream: &mut TcpStream) -> Option<Value> {
    let mut reader = BufReader::new(stream);
    let mut head = String::new();
    let mut length = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 || line == "\r\n" {
            break;
        }
        if let Some(size) = counted(&line) {
            length = size;
        }
        head.push_str(&line);
    }
    if head.starts_with("OPTIONS") || length == 0 {
        return Some(Value::Null);
    }
    let mut body = vec![0; length.min(LIMIT)];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn counted(line: &str) -> Option<usize> {
    let (name, value) = line.split_once(':')?;
    if !name.eq_ignore_ascii_case("content-length") {
        return None;
    }
    value.trim().parse().ok()
}

fn answer(stream: &mut TcpStream, status: u16, body: &str) {
    let head = format!(
        "HTTP/1.1 {status}\r\ncontent-type: application/json; charset=utf-8\r\n\
         access-control-allow-origin: *\r\naccess-control-allow-headers: content-type\r\n\
         access-control-allow-methods: POST, OPTIONS\r\ncontent-length: {}\r\n\
         connection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body.as_bytes());
}
