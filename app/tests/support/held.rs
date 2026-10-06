use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tolearn_app::ipc::{IpcError, call};

use super::speaking::speaking;
use super::starter::{Case, LEVEL, REQUEST, flat};

pub struct Held {
    pub case: Case,
    pub release: Sender<()>,
    pub drawing: JoinHandle<Result<Value, IpcError>>,
}

pub fn held() -> Held {
    let (release, waiting) = mpsc::channel::<()>();
    let answers = flat();
    let model = speaking(move |_, turn| {
        if turn == 0 {
            let _ = waiting.recv_timeout(Duration::from_secs(20));
        }
        answers[0].clone()
    });
    let case = Case::new(true, model);
    let context = case.context.clone();
    let drawing = std::thread::spawn(move || call(&context, "plan_program", &asked()));
    let until = Instant::now() + Duration::from_secs(20);
    while case.model.heard().is_empty() {
        assert!(Instant::now() < until, "запрос карты не дошёл до модели");
        std::thread::sleep(Duration::from_millis(10));
    }
    Held {
        case,
        release,
        drawing,
    }
}

pub fn asked() -> Value {
    json!({ "request": REQUEST, "level": LEVEL, "locale": "ru" })
}

pub fn state(case: &Case) -> Value {
    call(&case.context, "generation_state", &json!({})).unwrap()
}

pub fn seen(case: &Case) -> Value {
    call(&case.context, "generation_seen", &json!({})).unwrap()
}
