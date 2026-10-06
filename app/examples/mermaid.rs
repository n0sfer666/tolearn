#![allow(
    clippy::expect_used,
    reason = "app example: a panic here is the report"
)]

use std::time::{Duration, Instant};

use tauri::Manager;
use tolearn_app::mermaid::{self, Drawing, Mermaid};
use tolearn_generate::diagram::{Diagram, draw};

const FLOW: &str = "flowchart LR\n  tape[Лента] --> head[Головка]\n  head --> table{Таблица переходов}\n  table -->|пишет символ| tape";
const SEQUENCE: &str = "sequenceDiagram\n  participant S as Ученик\n  participant A as Приложение\n  S->>A: ответ на вопрос\n  A-->>S: разбор и следующий шаг";
const BROKEN: &str = "flowchart LR\n  A[Лента --> ";

fn main() {
    tauri::Builder::default()
        .plugin(mermaid::plugin())
        .setup(|app| {
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
                let failures = check(&handle);
                handle.exit(i32::from(failures > 0));
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("окно приложения не поднялось");
}

fn check<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> usize {
    let painter = Mermaid::new(app.clone(), Drawing::default());
    let mut failures = 0;
    for (name, source) in [("flowchart", FLOW), ("sequence", SEQUENCE)] {
        let started = Instant::now();
        failures += match draw(&painter, source) {
            Diagram::Drawn(drawn) => {
                let path = std::env::temp_dir().join(format!("tolearn-mermaid-{name}.svg"));
                let saved = std::fs::write(&path, &drawn.bytes).is_ok();
                println!(
                    "     {} байт, {:?}, {}",
                    drawn.bytes.len(),
                    started.elapsed(),
                    path.display()
                );
                report(&format!("{name} нарисована и прошла проверку SVG"), saved)
            }
            Diagram::Source { reason, .. } => {
                report(&format!("{name} нарисована: {reason}"), false)
            }
        };
    }
    failures += match draw(&painter, BROKEN) {
        Diagram::Source { block, reason } => {
            println!("     {reason}");
            report(
                "битая схема осталась исходником",
                block.kind.label() == "code",
            )
        }
        Diagram::Drawn(_) => report("битая схема осталась исходником", false),
    };
    let hasty = Mermaid::new(
        app.clone(),
        Drawing {
            timeout: Duration::from_millis(1),
            poll: Duration::from_millis(1),
        },
    );
    failures += match draw(&hasty, FLOW) {
        Diagram::Source { reason, .. } => report(&format!("таймаут: {reason}"), true),
        Diagram::Drawn(_) => report("таймаут отдаёт исходник", false),
    };
    failures + report("окна схем закрыты", closed(app, Duration::from_secs(2)))
}

fn closed<R: tauri::Runtime>(app: &tauri::AppHandle<R>, patience: Duration) -> bool {
    let until = Instant::now() + patience;
    loop {
        let gone = app
            .webview_windows()
            .keys()
            .all(|label| !label.starts_with("mermaid-"));
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
