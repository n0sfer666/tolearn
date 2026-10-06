use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use serde_json::{Value, json};
use tolearn_app::ipc::{Context, GenerationStep, IpcError, Tools, call};
use tolearn_provider::{Remembered, Vault};

use super::planner::{Net, provider};
use super::repository;
use super::speaking::{Speaking, speaking};
use super::web::{Canvas, Web};

pub const REQUEST: &str = "Хочу писать чиптюн";
pub const LEVEL: &str = "Нот не знаю";
const STORE: [&str; 2] = ["objects", "artifacts"];
const LONG: &str = "Скважность импульса меняет тембр. ";

static CASES: AtomicUsize = AtomicUsize::new(0);

type Heard = Arc<Mutex<Vec<(String, String)>>>;

pub struct Case {
    pub context: Context,
    pub data: PathBuf,
    pub model: Speaking,
    heard: Heard,
}

impl Case {
    pub fn new(up: bool, model: Speaking) -> Self {
        let data = super::scratch::made(&format!(
            "start-ipc-{}",
            CASES.fetch_add(1, Ordering::Relaxed)
        ));
        let heard = Heard::default();
        let seen = Arc::clone(&heard);
        let tools = Tools {
            source: Some(Arc::new(Web)),
            renderer: None,
            painter: Some(Arc::new(Canvas)),
            herald: Some(Arc::new(move |step: GenerationStep| {
                seen.lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push((step.state, step.step));
            })),
        };
        let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
        let context = Context::with_vault(&data, vault)
            .with_reach(Arc::new(Net(up)))
            .with_tools(tools);
        let saved = json!({
            "save": provider(&model.endpoint),
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        });
        call(&context, "provider", &saved).unwrap();
        Self {
            context,
            data,
            model,
            heard,
        }
    }

    pub fn plan(&self) -> Value {
        let drawn = call(
            &self.context,
            "plan_program",
            &json!({ "request": REQUEST, "level": LEVEL, "locale": "ru" }),
        );
        drawn.unwrap()["plan"].clone()
    }

    pub fn start(&self, plan: &Value, level: &str) -> Result<Value, IpcError> {
        call(&self.context, "start_program", &started(plan, level))
    }

    pub fn cancel(&self) -> Value {
        call(&self.context, "cancel_generation", &json!({})).unwrap()
    }

    pub fn steps(&self) -> Vec<(String, String)> {
        self.heard
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn leftovers(&self) -> Vec<String> {
        let Ok(listing) = std::fs::read_dir(self.data.join("cache")) else {
            return Vec::new();
        };
        listing
            .map(|item| item.unwrap().path())
            .filter(|path| path.is_dir())
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .filter(|name| !STORE.contains(&name.as_str()))
            .collect()
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.data);
    }
}

pub fn started(plan: &Value, level: &str) -> Value {
    json!({ "request": REQUEST, "level": level, "locale": "ru", "plan": plan })
}

fn fixture(path: &str) -> String {
    std::fs::read_to_string(repository().join("fixtures").join(path)).unwrap()
}

pub fn fine() -> String {
    let text = fixture("generate/stage/stage.txt");
    let mut stage: Value = serde_json::from_str(&text[text.find('{').unwrap()..]).unwrap();
    let blocks = stage["blocks"].as_array_mut().unwrap();
    blocks[3]["text"] = json!("Импульсная волна со скважностью 25%");
    blocks.push(json!({ "kind": "paragraph", "text": LONG.repeat(100), "sources": ["b1"] }));
    stage.to_string()
}

pub fn flat() -> Vec<String> {
    vec![
        fixture("generate/plan/flat.txt"),
        fixture("generate/stage/sources.txt"),
        fine(),
    ]
}

pub fn told(answers: Vec<String>) -> Speaking {
    speaking(move |_, turn| answers[turn.min(answers.len() - 1)].clone())
}
