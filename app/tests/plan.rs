#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use support::planner::{Net, provider};
use support::speaking::{Speaking, speaking};
use support::{repository, snapshot};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_provider::{Remembered, Vault};

static CASES: AtomicUsize = AtomicUsize::new(0);

const WISHES: [&str; 3] = [
    "Без теории музыки",
    "Только Famitracker",
    "Короче, на выходные",
];

struct Case {
    context: Context,
    data: PathBuf,
    model: Speaking,
}

fn case(up: bool, answers: &[&str]) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-plan-ipc-{}-{}",
        std::process::id(),
        CASES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let answers: Vec<String> = answers.iter().map(|name| answer(name)).collect();
    let model = speaking(move |_, turn| answers[turn.min(answers.len() - 1)].clone());
    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    let context = Context::with_vault(&data, vault).with_reach(Arc::new(Net(up)));
    call(
        &context,
        "provider",
        &json!({
            "save": provider(&model.endpoint),
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();
    Case {
        context,
        data,
        model,
    }
}

fn answer(name: &str) -> String {
    std::fs::read_to_string(repository().join("fixtures/generate/plan").join(name)).unwrap()
}

fn plan(case: &Case, request: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        "plan_program",
        &json!({ "request": request, "level": "Нот не знаю" }),
    )
}

fn revise(case: &Case, plan: &Value, wish: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        "revise_plan",
        &json!({ "request": "Хочу писать чиптюн", "level": "Нот не знаю", "plan": plan, "wish": wish }),
    )
}

#[test]
fn карта_и_три_переделки_не_пишут_в_библиотеку_ни_байта() {
    let case = case(true, &["flat.txt"]);
    let before = snapshot(&case.data);

    let mut drawn = plan(&case, "Хочу писать чиптюн").unwrap();
    assert_eq!(drawn["plan"]["title"], json!("Чиптюн с нуля"));
    assert_eq!(drawn["plan"]["volatility"], json!("stable"));
    assert_eq!(drawn["hours"], json!({ "min": 19, "max": 30 }));
    for wish in WISHES {
        drawn = revise(&case, &drawn["plan"], wish).unwrap();
    }

    let heard = case.model.heard();
    assert_eq!(heard.len(), 4);
    assert!(heard[0].contains("Хочу писать чиптюн"), "{}", heard[0]);
    for (prompt, wish) in heard[1..].iter().zip(WISHES) {
        for asked in ["Прежняя карта", "\"first-track\"", "Нот не знаю", wish]
        {
            assert!(prompt.contains(asked), "{asked}: {prompt}");
        }
    }
    assert!(!case.data.join("programs").exists());
    assert_eq!(snapshot(&case.data), before);
}

#[test]
fn большая_карта_приходит_подпрограммами_с_суммой_часов() {
    let case = case(true, &["split.txt"]);

    let drawn = plan(&case, "Хочу разобраться с локальными LLM").unwrap();

    assert_eq!(drawn["plan"]["stages"], json!([]));
    assert_eq!(
        drawn["plan"]["children"][1],
        json!({
            "title": "Квантование и форматы",
            "goal": "Выбирать формат и квантование под своё железо.",
            "hours": { "min": 30, "max": 50 },
        })
    );
    assert_eq!(drawn["hours"], json!({ "min": 170, "max": 250 }));
}

#[test]
fn без_сети_карта_отказывает_с_причиной_и_модель_не_зовут() {
    let case = case(false, &["flat.txt"]);

    let refused = plan(&case, "Хочу писать чиптюн").unwrap_err();

    assert_eq!(refused.code, "generate.offline");
    assert!(
        refused.message.contains("openlibrary.org"),
        "{}",
        refused.message
    );
    assert!(case.model.heard().is_empty());
}

#[test]
fn пустой_запрос_уточнение_и_чужая_изменчивость_отказывают_до_модели() {
    let case = case(true, &["flat.txt"]);

    assert_eq!(plan(&case, "  ").unwrap_err().code, "plan.empty");
    let drawn = plan(&case, "Хочу писать чиптюн").unwrap();
    assert_eq!(
        revise(&case, &drawn["plan"], " ").unwrap_err().code,
        "plan.empty"
    );
    let mut odd = drawn["plan"].clone();
    odd["volatility"] = json!("forever");
    assert_eq!(
        revise(&case, &odd, "Короче").unwrap_err().code,
        "plan.unknown-value"
    );

    assert_eq!(case.model.heard().len(), 1);
}

#[test]
fn карта_без_починки_отказывает_списком_нарушений() {
    let case = case(true, &["broken.txt"]);

    let refused = plan(&case, "Хочу писать чиптюн").unwrap_err();

    assert_eq!(refused.code, "generate.unrepaired");
    assert!(
        refused.message.contains("«tracker» повторяется"),
        "{}",
        refused.message
    );
    assert_eq!(case.model.heard().len(), 4);
}
