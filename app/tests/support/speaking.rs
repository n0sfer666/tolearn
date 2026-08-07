use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

pub struct Speaking {
    pub endpoint: String,
    heard: Arc<Mutex<Vec<String>>>,
}

impl Speaking {
    pub fn heard(&self) -> Vec<String> {
        self.heard
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

pub fn speaking(answer: impl Fn(&str, usize) -> String + Send + 'static) -> Speaking {
    served(move |prompt, turn| Some(answer(prompt, turn)))
}

pub fn breaking(answer: impl Fn(&str, usize) -> Option<String> + Send + 'static) -> Speaking {
    served(answer)
}

fn served(answer: impl Fn(&str, usize) -> Option<String> + Send + 'static) -> Speaking {
    let listener = TcpListener::bind("127.0.0.1:0").expect("порт не даётся");
    let endpoint = format!("http://{}", listener.local_addr().expect("нет адреса"));
    let heard = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&heard);
    let turn = AtomicUsize::new(0);

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let asked = taken(&stream);
            let prompt = prompt(&asked);
            seen.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(prompt.clone());
            let Some(said) = answer(&prompt, turn.fetch_add(1, Ordering::Relaxed)) else {
                continue;
            };
            let body = serde_json::json!({
                "message": { "content": said },
                "eval_count": 11,
                "prompt_eval_count": 7,
            })
            .to_string();
            let mut writer = &stream;
            let _ = write!(
                writer,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = writer.flush();
        }
    });

    Speaking { endpoint, heard }
}

fn taken(stream: &std::net::TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        head.push_str(&line);
    }
    let mut body = vec![0; length(&head)];
    let _ = reader.read_exact(&mut body);
    String::from_utf8_lossy(&body).into_owned()
}

fn prompt(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|asked| asked["messages"][0]["content"].as_str().map(str::to_owned))
        .unwrap_or_default()
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
