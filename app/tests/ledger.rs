#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::speaking::breaking;
use support::starter::{Case, LEVEL, REQUEST, flat, told};
use tolearn_app::ipc::call;
use tolearn_generate::ledger::{self, Kind, Record};

const STAGE: [&str; 6] = [
    "sources", "sources", "sources", "sources", "sources", "text",
];

fn answers(plans: usize) -> Vec<String> {
    let flat = flat();
    let mut answers = vec![flat[0].clone(); plans];
    answers.extend(flat[1..].iter().cloned());
    answers
}

fn revised(case: &Case, plan: &Value) -> Value {
    let wish = json!({ "request": REQUEST, "level": LEVEL, "locale": "ru", "plan": plan, "wish": "Короче" });
    call(&case.context, "revise_plan", &wish).unwrap()["plan"].clone()
}

fn written(case: &Case, plan: &Value) -> (String, Vec<Record>) {
    let started = case.start(plan, LEVEL).unwrap();
    let program = started["program"].as_str().unwrap().to_owned();
    let records = ledger::read(&ledger::path(&case.data, &program)).unwrap();
    (program, records)
}

fn steps(records: &[Record]) -> Vec<&str> {
    records.iter().map(|record| record.step.as_str()).collect()
}

#[test]
fn карта_и_переделка_ложатся_в_журнал_программы_первыми() {
    let case = Case::new(true, told(answers(2)));
    let plan = revised(&case, &case.plan());

    let (program, records) = written(&case, &plan);

    let mut expected = vec!["plan", "revise"];
    expected.extend(STAGE);
    assert_eq!(steps(&records), expected);
    assert!(
        records
            .iter()
            .all(|record| record.program.as_deref() == Some(program.as_str())),
        "{records:?}"
    );
    for record in records.iter().filter(|record| record.kind == Kind::Model) {
        assert_eq!((record.input, record.output), (Some(7), Some(11)));
        assert_eq!(record.model.as_deref(), Some("llama3:8b"));
    }
}

#[test]
fn новая_карта_сбрасывает_накопленное_а_удачный_старт_забирает_его() {
    let case = Case::new(true, told(answers(2)));
    case.plan();
    let plan = case.plan();

    let (_, records) = written(&case, &plan);

    let mut expected = vec!["plan"];
    expected.extend(STAGE);
    assert_eq!(steps(&records), expected);
    assert!(case.context.ledger().is_empty());
}

#[test]
fn отказ_новой_карты_дописывает_накопленное_а_не_стирает() {
    let case = Case::new(true, told(vec![flat()[0].clone(), "не карта".to_owned()]));
    case.plan();
    let kept = case.context.ledger().records();

    let empty = call(
        &case.context,
        "plan_program",
        &json!({ "request": " ", "level": LEVEL, "locale": "ru" }),
    );
    assert_eq!(empty.unwrap_err().code, "plan.empty");
    assert_eq!(case.context.ledger().records(), kept);

    let broken = call(
        &case.context,
        "plan_program",
        &json!({ "request": REQUEST, "level": LEVEL, "locale": "ru" }),
    );
    assert!(broken.is_err());
    let records = case.context.ledger().records();
    assert_eq!(records[..1], kept[..]);
    let rounds: Vec<Option<usize>> = records.iter().map(|record| record.round).collect();
    assert_eq!(rounds, [None, None, Some(1), Some(2), Some(3)]);
    assert!(
        records.iter().all(|record| record.step == "plan"),
        "{records:?}"
    );
}

#[test]
fn упавший_старт_возвращает_попытку_в_память_а_повтор_пишет_её_без_этапа() {
    let flat = flat();
    let case = Case::new(
        true,
        breaking(move |_, turn| match turn {
            0 | 1 => Some(flat[0].clone()),
            2 | 4 => Some(flat[1].clone()),
            3 => None,
            _ => Some(flat[2].clone()),
        }),
    );
    let plan = revised(&case, &case.plan());

    case.start(&plan, LEVEL).unwrap_err();

    let kept = case.context.ledger().records();
    let mut expected = vec!["plan", "revise"];
    expected.extend(STAGE);
    assert_eq!(steps(&kept), expected);
    assert!(!kept.last().unwrap().ok);
    assert!(
        kept.iter().all(|record| record.program.is_none()
            && record.stage.is_none()
            && record.at.is_none()),
        "{kept:?}"
    );

    let (program, records) = written(&case, &plan);

    expected.extend(STAGE);
    assert_eq!(steps(&records), expected);
    let first = plan["stages"][0]["id"].as_str().unwrap();
    let stages: Vec<Option<&str>> = records
        .iter()
        .map(|record| record.stage.as_deref())
        .collect();
    assert!(stages[..8].iter().all(Option::is_none), "{stages:?}");
    assert!(
        stages[8..].iter().all(|stage| *stage == Some(first)),
        "{stages:?}"
    );
    assert!(
        records.iter().all(
            |record| record.at.is_some() && record.program.as_deref() == Some(program.as_str())
        ),
        "{records:?}"
    );
    assert!(case.context.ledger().is_empty());
}
