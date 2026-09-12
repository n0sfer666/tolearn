#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::repository;
use support::speaking::{Speaking, speaking};
use support::starter::{Case, LEVEL, started};
use support::web::{SVG, THUMBNAIL};
use tolearn_app::ipc::call;
use tolearn_core::block::Kind;
use tolearn_core::library::Library;

const LONG: &str = "Скважность импульса меняет тембр. ";

fn fixture(path: &str) -> String {
    std::fs::read_to_string(repository().join("fixtures").join(path)).unwrap()
}

fn fine() -> String {
    let text = fixture("generate/stage/stage.txt");
    let mut stage: Value = serde_json::from_str(&text[text.find('{').unwrap()..]).unwrap();
    let blocks = stage["blocks"].as_array_mut().unwrap();
    blocks[3]["text"] = json!("Импульсная волна со скважностью 25%");
    blocks.push(json!({ "kind": "paragraph", "text": LONG.repeat(100), "sources": ["b1"] }));
    stage.to_string()
}

fn flat() -> Vec<String> {
    vec![
        fixture("generate/plan/flat.txt"),
        fixture("generate/stage/sources.txt"),
        fine(),
    ]
}

fn told(answers: Vec<String>) -> Speaking {
    speaking(move |_, turn| answers[turn.min(answers.len() - 1)].clone())
}

fn paired(steps: &[&str]) -> Vec<(String, String)> {
    steps
        .iter()
        .flat_map(|step| ["began", "ended"].map(|state| (state.to_owned(), (*step).to_owned())))
        .collect()
}

#[test]
fn начать_ставит_программу_с_первым_этапом_схемой_и_картинкой() {
    let case = Case::new(true, told(flat()));
    let plan = case.plan();

    let started = case.start(&plan, LEVEL).unwrap();
    let program = started["program"].as_str().unwrap();

    let library = Library::at(&case.data);
    let tree = library.open(program).unwrap();
    assert_eq!(tree.program.level, LEVEL);
    assert_eq!(json!(tree.program.title), plan["title"]);
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["tracker"]);
    let stage = &tree.stages["tracker"];
    let diagram = stage
        .every_block()
        .find(|block| block.kind == Kind::Diagram)
        .unwrap();
    let svg = diagram.asset.as_deref().unwrap();
    assert_eq!(library.asset(&tree, svg).unwrap(), SVG.as_bytes());
    let image = stage
        .every_block()
        .find(|block| block.kind == Kind::Image)
        .unwrap();
    assert!(image.license.is_some() && image.attribution.is_some());
    let png = image.asset.as_deref().unwrap();
    assert_eq!(library.asset(&tree, png).unwrap(), THUMBNAIL);

    let read = call(
        &case.context,
        "stage",
        &json!({ "program": program, "node": "", "stage": "tracker" }),
    );
    assert_eq!(read.unwrap()["program"], json!(program));
    assert_eq!(
        case.steps(),
        paired(&["sources", "text", "diagrams", "write"])
    );
    assert_eq!(case.model.heard().len(), 3);
    assert_eq!(case.leftovers(), [program]);
    assert!(!case.data.join("cache").join(program).join("build").exists());
}

#[test]
fn без_уровня_с_чужой_или_негодной_картой_старт_отказывает_до_модели() {
    let case = Case::new(true, told(flat()));
    let plan = case.plan();
    let mut odd = plan.clone();
    odd["volatility"] = json!("forever");
    let mut unfit = plan.clone();
    unfit["title"] = json!("");

    assert_eq!(case.start(&plan, " ").unwrap_err().code, "plan.empty");
    assert_eq!(
        case.start(&odd, LEVEL).unwrap_err().code,
        "plan.unknown-value"
    );
    assert_eq!(
        case.start(&unfit, LEVEL).unwrap_err().code,
        "generate.unfit"
    );

    assert_eq!(case.model.heard().len(), 1);
    assert!(case.steps().is_empty());
    assert!(!case.data.join("programs").exists());
}

#[test]
fn без_сети_старт_отказывает_с_причиной_и_модель_не_зовут() {
    let plan = Case::new(true, told(flat())).plan();
    let case = Case::new(false, told(flat()));

    let refused = case.start(&plan, LEVEL).unwrap_err();

    assert_eq!(refused.code, "generate.offline");
    assert!(case.model.heard().is_empty());
    assert!(!case.data.join("programs").exists());
    assert_eq!(case.cancel(), json!({ "cancelled": false }));
}

#[test]
fn вторая_генерация_занята_а_отмена_снимает_первую_без_следа() {
    let (release, held) = mpsc::channel::<()>();
    let answers = flat();
    let model = speaking(move |_, turn| {
        if turn == 1 {
            let _ = held.recv_timeout(Duration::from_secs(20));
        }
        answers[turn.min(answers.len() - 1)].clone()
    });
    let case = Case::new(true, model);
    let plan = case.plan();
    let context = case.context.clone();
    let payload = started(&plan, LEVEL);
    let first = std::thread::spawn(move || call(&context, "start_program", &payload));
    let until = Instant::now() + Duration::from_secs(20);
    while case.model.heard().len() < 2 {
        assert!(
            Instant::now() < until,
            "запрос источников не дошёл до модели"
        );
        std::thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(case.start(&plan, LEVEL).unwrap_err().code, "generate.busy");
    assert_eq!(case.cancel(), json!({ "cancelled": true }));
    let refused = first.join().unwrap().unwrap_err();
    let _ = release.send(());

    assert_eq!(refused.code, "generate.cancelled");
    assert_eq!(case.cancel(), json!({ "cancelled": false }));
    assert_eq!(case.steps(), paired(&["sources"]));
    assert!(!case.data.join("programs").exists());
    assert!(case.leftovers().is_empty(), "{:?}", case.leftovers());
}
