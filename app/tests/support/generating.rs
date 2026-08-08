use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_provider::{Remembered, Vault};

use super::speaking::Speaking;

static CASES: AtomicUsize = AtomicUsize::new(0);

pub const TODAY: &str = "2026-08-06";
pub const SLOW: Duration = Duration::from_secs(3);

pub struct Case {
    pub context: Context,
    pub data: PathBuf,
}

pub fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-generate-{name}-{}-{}",
        std::process::id(),
        CASES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    Case {
        context: Context::with_vault(&data, vault),
        data,
    }
}

pub fn enable(case: &Case, endpoint: &str) {
    call(
        &case.context,
        "provider",
        &json!({
            "save": {
                "enabled": true,
                "active": "local",
                "journal": false,
                "local": {
                    "endpoint": endpoint,
                    "api": "ollama",
                    "model": "llama3:8b",
                    "num_ctx": 0,
                    "temperature_tenths": 7,
                },
                "remote": {
                    "endpoint": endpoint,
                    "api": "openai",
                    "model": "gpt",
                    "num_ctx": 0,
                    "temperature_tenths": 7,
                },
                "harness": {
                    "id": "claude",
                    "command": "claude",
                    "args": ["-p"],
                    "timeout_secs": 180,
                },
            },
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();
}

pub fn started(case: &Case, heard: &Speaking) -> String {
    enable(case, &heard.endpoint);
    let out = call(
        &case.context,
        "generate",
        &json!({
            "subject": "поднять локальную модель",
            "level": "basics",
            "weekly_hours": 6,
            "weeks": 10,
            "today": TODAY,
        }),
    )
    .unwrap();
    out["job"].as_str().unwrap().to_owned()
}

pub fn go(case: &Case, job: &str) {
    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
}

pub fn until(case: &Case, job: &str, ready: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let live = state(case, job);
        if ready(&live) {
            return live;
        }
        assert!(Instant::now() < deadline, "генерация застряла: {live}");
        sleep(Duration::from_millis(20));
    }
}

pub fn finished(case: &Case, job: &str) -> Value {
    until(case, job, |live| live["finished"] == json!(true))
}

pub fn waiting(case: &Case, job: &str) -> Value {
    until(case, job, |live| live["waiting"] == json!(true))
}

pub fn state(case: &Case, job: &str) -> Value {
    call(&case.context, "generate_state", &json!({ "job": job })).unwrap()
}

pub fn accept(case: &Case, job: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        "generate_accept",
        &json!({ "job": job, "today": TODAY }),
    )
}

pub fn draft(case: &Case, take: bool) -> Value {
    call(
        &case.context,
        "generate_draft",
        &json!({ "take": take, "drop": false, "today": TODAY }),
    )
    .unwrap()
}

pub fn bundles(case: &Case) -> PathBuf {
    case.data.join("unpacked")
}
