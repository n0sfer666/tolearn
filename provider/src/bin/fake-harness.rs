use std::io::Read;
use std::process::ExitCode;
use std::time::Duration;

const VERSION: &str = "fake-harness 1.0";
const FLOOD_LINES: usize = 40_000;
const STEP: Duration = Duration::from_millis(40);
const SPENT: &str = r#"{"input_tokens":10,"cache_creation_input_tokens":4,"output_tokens":7,"cache_read_input_tokens":3}"#;
const UNSPENT: &str = r#"{"input_tokens":0,"output_tokens":0}"#;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map_or("say", String::as_str);
    match mode {
        "--version" => println!("{VERSION}"),
        "say" => println!("услышал: {}", heard()),
        "ansi" => println!("\u{1b}]0;title\u{7}\u{1b}[32mуслышал: {}\u{1b}[0m", heard()),
        "banner" => println!("добро пожаловать\n\nуслышал: {}", heard()),
        "silent" => {}
        "stream" => stream(&heard(), SPENT),
        "unspent" => stream(&heard(), UNSPENT),
        "jsonish" => println!(
            "{{\"type\":\"status\",\"note\":\"работаю\"}}\nуслышал: {}",
            heard()
        ),
        "args" => println!("{}", args[1..].join(" ")),
        "cwd" => println!("{}", around()),
        "flood" => flood(),
        "hang" => std::thread::sleep(Duration::from_secs(30)),
        "linger" => linger(&args[1]),
        "fail" => {
            eprintln!("не залогинен\nвыполните `fake-harness login`\nстрока три\nстрока четыре");
            return ExitCode::from(3);
        }
        other => {
            eprintln!("не знаю режим {other}");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}

fn stream(said: &str, usage: &str) {
    println!(r#"{{"type":"system","subtype":"init","model":"fake-sonnet"}}"#);
    for piece in ["услышал", ": ", said] {
        println!(
            r#"{{"type":"stream_event","event":{{"type":"content_block_delta","delta":{{"type":"text_delta","text":"{piece}"}}}}}}"#
        );
        std::thread::sleep(STEP);
    }
    println!(
        r#"{{"type":"result","subtype":"success","result":"услышал: {said}","usage":{usage}}}"#
    );
}

fn heard() -> String {
    let mut said = String::new();
    let _ = std::io::stdin().read_to_string(&mut said);
    said.trim().to_owned()
}

fn around() -> String {
    let Ok(here) = std::env::current_dir() else {
        return "нет каталога".to_owned();
    };
    let count = std::fs::read_dir(&here).map_or(0, |entries| entries.count());
    format!("{} файлов {}", count, here.display())
}

fn flood() {
    for line in 0..FLOOD_LINES {
        println!("строка {line} набита буквами чтобы вывод перевалил за потолок");
    }
}

fn linger(pid: &str) {
    let _ = std::fs::write(pid, std::process::id().to_string());
    std::thread::sleep(Duration::from_secs(30));
}
