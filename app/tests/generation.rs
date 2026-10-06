#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::sync::Arc;

use serde_json::{Value, json};
use support::bucket::Bucket;
use support::held::{held, seen, state};
use support::starter::{Case, LEVEL, REQUEST, flat, told};
use tolearn_app::ipc::{Finale, GenerationWork, call, generated};

fn steps(work: &Value) -> Vec<(String, String)> {
    work["marks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|mark| {
            assert!(mark["at"].as_u64().unwrap() >= work["began"].as_u64().unwrap());
            (
                mark["step"]["state"].as_str().unwrap().to_owned(),
                mark["step"]["step"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn asked(work: &Value, kind: &str) {
    assert_eq!(work["kind"], json!(kind));
    assert_eq!(work["request"], json!(REQUEST));
    assert_eq!(work["level"], json!(LEVEL));
    assert_eq!(work["locale"], json!("ru"));
    assert_eq!(work["program"], json!(""));
}

#[test]
fn карта_без_слушателя_ждёт_в_состоянии_вместе_с_запросом() {
    let case = Case::new(true, told(flat()));

    let plan = case.plan();

    let state = state(&case);
    assert_eq!(state["work"], Value::Null);
    let outcome = &state["outcome"];
    asked(&outcome["work"], "plan");
    assert_eq!(outcome["plan"]["plan"], plan);
    assert_eq!(outcome["refusal"], Value::Null);
    assert_eq!(outcome["seen"], json!(false));
    assert_eq!(
        steps(&outcome["work"]),
        [
            ("began".into(), "plan".into()),
            ("ended".into(), "plan".into())
        ]
    );
}

#[test]
fn идущая_карта_отдаёт_запрос_и_шаги_до_конца() {
    let held = held();

    let during = state(&held.case);
    held.release.send(()).unwrap();
    let plan = held.drawing.join().unwrap().unwrap();

    asked(&during["work"], "plan");
    assert_eq!(during["outcome"], Value::Null);
    assert_eq!(steps(&during["work"]), [("began".into(), "plan".into())]);
    let after = state(&held.case);
    assert_eq!(after["work"], Value::Null);
    assert_eq!(after["outcome"]["plan"], plan);
}

#[test]
fn отказ_карты_хранит_причину() {
    let case = Case::new(false, told(flat()));

    let refused = call(&case.context, "plan_program", &support::held::asked()).unwrap_err();

    let outcome = &state(&case)["outcome"];
    assert_eq!(refused.code, "generate.offline");
    assert_eq!(
        outcome["refusal"],
        json!({ "code": refused.code, "message": refused.message })
    );
    assert_eq!(outcome["plan"], Value::Null);
    asked(&outcome["work"], "plan");
}

#[test]
fn отмена_карты_отвечает_true_и_оставляет_пустое_состояние() {
    let held = held();

    assert_eq!(held.case.cancel(), json!({ "cancelled": true }));
    let refused = held.drawing.join().unwrap().unwrap_err();
    let _ = held.release.send(());

    assert_eq!(refused.code, "generate.cancelled");
    assert_eq!(state(&held.case), json!({ "work": null, "outcome": null }));
    assert_eq!(held.case.cancel(), json!({ "cancelled": false }));
}

#[test]
fn увиденная_карта_ждёт_а_увиденный_этап_снимается() {
    let case = Case::new(true, told(flat()));
    let plan = case.plan();

    let after = seen(&case);

    assert_eq!(after["outcome"]["seen"], json!(true));
    assert_eq!(after["outcome"]["plan"]["plan"], plan);
    assert_eq!(state(&case), after);

    let started = case.start(&plan, LEVEL).unwrap();
    let outcome = &state(&case)["outcome"];
    asked(&outcome["work"], "start");
    assert_eq!(outcome["work"]["plan"], plan);
    assert_eq!(outcome["plan"], Value::Null);
    assert_eq!(outcome["stage"], started);

    assert_eq!(seen(&case), json!({ "work": null, "outcome": null }));
}

#[test]
fn удаление_программы_снимает_её_итог() {
    let mut case = Case::new(true, told(flat()));
    let bucket = Bucket::new(&case.data.join("bin"), None);
    case.context = case.context.clone().with_bin(Arc::new(bucket));
    let program = case.start(&case.plan(), LEVEL).unwrap()["program"].clone();
    assert_eq!(state(&case)["outcome"]["stage"]["program"], program);

    call(
        &case.context,
        "delete_program",
        &json!({ "program": program }),
    )
    .unwrap();

    assert_eq!(state(&case), json!({ "work": null, "outcome": null }));
}

#[test]
fn неудачное_удаление_оставляет_итог_программы() {
    let mut case = Case::new(true, told(flat()));
    let started = case.start(&case.plan(), LEVEL).unwrap();
    let program = started["program"].as_str().unwrap().to_owned();
    let bucket = Bucket::new(&case.data.join("bin"), Some("programs"));
    case.context = case.context.clone().with_bin(Arc::new(bucket));
    let before = state(&case);

    let refused = call(
        &case.context,
        "delete_program",
        &json!({ "program": program }),
    )
    .unwrap_err();

    assert_eq!(refused.code, "delete.left");
    assert_eq!(before["outcome"]["stage"]["program"], json!(program));
    assert_eq!(state(&case), before);
}

#[test]
fn принятая_отмена_при_готовом_ответе_итога_не_оставляет() {
    let case = Case::new(true, told(flat()));
    let running = case.context.running();
    let mut accepted = false;

    let refused = generated(
        &case.context,
        GenerationWork::asked("plan", REQUEST, LEVEL, "ru"),
        |_| {
            accepted = running.cancel();
            Ok(())
        },
        |()| Finale::Crashed,
    )
    .unwrap_err();

    assert!(accepted);
    assert_eq!(refused.code, "generate.cancelled");
    assert_eq!(state(&case), json!({ "work": null, "outcome": null }));
}
