#![allow(
    clippy::expect_used,
    reason = "app example: a panic here is the report"
)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use tauri::Manager;
use tolearn_app::prerender::{Settling, Webview};
use tolearn_offline::page::Prerenderer;

const PAGE: &str = "<!doctype html><html lang=\"ru\"><head><meta charset=\"utf-8\">\
<title>Пусто до JS</title></head><body><div id=\"app\"></div>\
<script>setTimeout(() => { document.getElementById('app').textContent = 'после JS'; }, 300);\
</script></body></html>";

fn main() {
    let address = serve();
    let url = format!("http://{address}/page");
    let fetched = PAGE.as_bytes().to_vec();

    tauri::Builder::default()
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
            let handle = app.handle().clone();
            let watchdog = handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(60));
                println!("НЕТ  проверка не уложилась в минуту");
                watchdog.exit(2);
            });
            std::thread::spawn(move || {
                let failures = check(&handle, &url, &fetched);
                handle.exit(i32::from(failures > 0));
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("окно приложения не поднялось");
}

fn check<R: tauri::Runtime>(app: &tauri::AppHandle<R>, url: &str, fetched: &[u8]) -> usize {
    let mut failures = 0;

    let settled = Webview::new(
        app.clone(),
        Settling {
            settle: Duration::from_millis(600),
            timeout: Duration::from_secs(10),
            poll: Duration::from_millis(100),
        },
    );
    let rendered = settled.render(url, fetched).expect("пререндер не открылся");
    let html = String::from_utf8_lossy(&rendered).into_owned();
    failures += report(
        "страница с клиентским рендерингом сохранена с содержимым",
        html.contains("после JS"),
    );

    let hasty = Webview::new(
        app.clone(),
        Settling {
            settle: Duration::from_secs(30),
            timeout: Duration::from_millis(700),
            poll: Duration::from_millis(100),
        },
    );
    let started = Instant::now();
    let fallback = hasty.render(url, fetched).expect("пререндер не открылся");
    let waited = started.elapsed();
    failures += report(
        "таймаут отдаёт скачанное и не вешает очередь",
        fallback == fetched && waited < Duration::from_secs(3),
    );
    failures += report(
        "окна пререндера закрыты",
        closed(app, Duration::from_secs(2)),
    );

    println!("ожидание после таймаута: {waited:?}");
    failures
}

fn closed<R: tauri::Runtime>(app: &tauri::AppHandle<R>, patience: Duration) -> bool {
    let until = Instant::now() + patience;
    loop {
        let gone = app
            .webview_windows()
            .keys()
            .all(|label| !label.starts_with("prerender-"));
        if gone || Instant::now() >= until {
            return gone;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn report(what: &str, passed: bool) -> usize {
    println!("{} {what}", if passed { "ок  " } else { "НЕТ " });
    usize::from(!passed)
}

fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("порт не занялся");
    let address = listener
        .local_addr()
        .expect("у сокета нет адреса")
        .to_string();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            answer(stream);
        }
    });
    address
}

fn answer(mut stream: TcpStream) {
    let mut request = [0; 1024];
    let _ = stream.read(&mut request);
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{PAGE}",
        PAGE.len()
    );
    let _ = stream.write_all(response.as_bytes());
}
