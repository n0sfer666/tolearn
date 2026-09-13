#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{Value, json};
use support::planner::{Net, provider};
use support::starter::{Case, LEVEL, flat, told};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_generate::ledger;

fn begun(extra: &[&str]) -> (Case, String) {
    let mut answers = flat();
    answers.extend(extra.iter().map(|&text| text.to_owned()));
    let case = Case::new(true, told(answers));
    let started = case.start(&case.plan(), LEVEL).unwrap();
    let program = started["program"].as_str().unwrap().to_owned();
    (case, program)
}

fn at(program: &str) -> Value {
    json!({ "program": program, "node": "", "stage": "tracker" })
}

fn shown(context: &Context, program: &str) -> Value {
    call(context, "stage", &at(program)).unwrap()
}

fn block(context: &Context, program: &str, kind: &str) -> Value {
    shown(context, program)["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["kind"] == json!(kind))
        .unwrap()
        .clone()
}

fn asked(
    context: &Context,
    program: &str,
    block: &str,
    question: &str,
    chain: Option<u32>,
) -> Result<Value, IpcError> {
    let mut input = at(program);
    input["block"] = json!(block);
    input["question"] = json!(question);
    input["chain"] = json!(chain);
    call(context, "clarify", &input)
}

fn chained(context: &Context, name: &str, program: &str, chain: u32) -> Result<Value, IpcError> {
    let mut input = at(program);
    input["chain"] = json!(chain);
    call(context, name, &input)
}

fn state(case: &Case, program: &str) -> PathBuf {
    case.data.join("state").join(program).join("state.yaml")
}

fn active(context: &Context, case: &Case, kind: &str) {
    let mut saved = provider(&case.model.endpoint);
    saved["active"] = json!(kind);
    let input = json!({ "save": saved, "key": Value::Null, "forget": false, "check": false, "probe": false });
    call(context, "provider", &input).unwrap();
}

#[test]
fn уточнение_уходит_одним_запросом_и_ложится_цепочкой_под_блок() {
    let (case, program) = begun(&["Пульс — это волна.\n\n> Врезка"]);
    let paragraph = block(&case.context, &program, "paragraph");
    let id = paragraph["id"].as_str().unwrap();
    let text = paragraph["text"].as_str().unwrap();
    let heard = case.model.heard().len();

    let result = asked(&case.context, &program, id, "  ", None).unwrap();

    let excerpt: String = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect();
    let expected = json!([{
        "chain": 0,
        "block": id,
        "excerpt": excerpt,
        "turns": [{ "asked": null, "answer": "Пульс — это волна.\n\n> Врезка" }],
        "clear": false,
    }]);
    assert_eq!(result["clarifications"], expected);
    let prompts = case.model.heard();
    assert_eq!(prompts.len(), heard + 1);
    assert!(prompts[heard].contains(text));
    assert!(prompts[heard].contains(LEVEL));
    let steps = case.steps();
    assert!(steps.contains(&("began".to_owned(), "clarify".to_owned())));
    assert!(steps.contains(&("ended".to_owned(), "clarify".to_owned())));
    let records = ledger::read(&ledger::path(&case.data, &program)).unwrap();
    let clarify: Vec<_> = records
        .iter()
        .filter(|record| record.step == "clarify")
        .collect();
    assert_eq!(clarify.len(), 1);
    assert_eq!(clarify[0].stage.as_deref(), Some("tracker"));
    assert_eq!(shown(&case.context, &program)["clarifications"], expected);
}

#[test]
fn нет_продолжает_цепочку_да_закрывает_а_новое_уточнение_ложится_ниже() {
    let (case, program) = begun(&["Первое.", "Второе.", "Третье."]);
    let paragraph = block(&case.context, &program, "paragraph");
    let id = paragraph["id"].as_str().unwrap();
    asked(&case.context, &program, id, "", None).unwrap();

    let result = asked(&case.context, &program, id, "А скважность?", Some(0)).unwrap();

    let prompts = case.model.heard();
    let last = prompts.last().unwrap();
    assert!(last.contains("Первое."));
    assert!(last.contains("А скважность?"));
    assert_eq!(
        result["clarifications"][0]["turns"],
        json!([
            { "asked": null, "answer": "Первое." },
            { "asked": "А скважность?", "answer": "Второе." },
        ])
    );

    let closed = chained(&case.context, "understood", &program, 0).unwrap();
    assert_eq!(closed["clarifications"][0]["clear"], json!(true));
    let heard = case.model.heard().len();
    let refused = asked(&case.context, &program, id, "Ещё", Some(0)).unwrap_err();
    assert_eq!(refused.code, "clarification.absent");
    assert_eq!(case.model.heard().len(), heard);

    let again = asked(&case.context, &program, id, "", None).unwrap();
    let chains: Vec<(u64, bool)> = again["clarifications"]
        .as_array()
        .unwrap()
        .iter()
        .map(|chain| {
            (
                chain["chain"].as_u64().unwrap(),
                chain["clear"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(chains, [(0, true), (1, false)]);
    assert_eq!(
        again["clarifications"][1]["turns"][0]["answer"],
        json!("Третье.")
    );
}

#[test]
fn убранное_уточнение_пропадает_а_чужой_номер_отказывает() {
    let (case, program) = begun(&["Объяснение."]);
    let id = block(&case.context, &program, "paragraph")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    asked(&case.context, &program, &id, "", None).unwrap();

    let removed = chained(&case.context, "unclarify", &program, 0).unwrap();

    assert_eq!(removed["clarifications"], json!([]));
    let again = chained(&case.context, "unclarify", &program, 0).unwrap_err();
    assert_eq!(again.code, "clarification.absent");
    let stray = chained(&case.context, "understood", &program, 5).unwrap_err();
    assert_eq!(stray.code, "clarification.absent");
    assert_eq!(shown(&case.context, &program)["clarifications"], json!([]));
}

#[test]
fn заголовок_и_чужой_блок_не_зовут_модель() {
    let (case, program) = begun(&["Объяснение."]);
    let heading = block(&case.context, &program, "heading");
    let heard = case.model.heard().len();

    let refused = asked(
        &case.context,
        &program,
        heading["id"].as_str().unwrap(),
        "",
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "clarification.block");
    let stray = asked(&case.context, &program, "nope", "", None).unwrap_err();
    assert_eq!(stray.code, "clarification.block");

    assert_eq!(case.model.heard().len(), heard);
    assert_eq!(shown(&case.context, &program)["clarifications"], json!([]));
}

#[test]
fn сеть_проверяется_только_у_удалённого_провайдера() {
    let (case, program) = begun(&["Объяснение."]);
    let id = block(&case.context, &program, "paragraph")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let offline = case.context.clone().with_reach(Arc::new(Net(false)));
    let heard = case.model.heard().len();

    active(&offline, &case, "remote");
    let refused = asked(&offline, &program, &id, "", None).unwrap_err();
    assert_eq!(refused.code, "generate.offline");
    assert_eq!(case.model.heard().len(), heard);

    active(&offline, &case, "local");
    let result = asked(&offline, &program, &id, "", None).unwrap();
    assert_eq!(
        result["clarifications"][0]["turns"][0]["answer"],
        json!("Объяснение.")
    );
}

#[test]
fn пустое_объяснение_отказывает_и_не_трогает_состояние() {
    let (case, program) = begun(&[" \n "]);
    let id = block(&case.context, &program, "paragraph")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = std::fs::read(state(&case, &program)).unwrap();

    let refused = asked(&case.context, &program, &id, "", None).unwrap_err();

    assert_eq!(refused.code, "provider.bad-answer");
    assert_eq!(std::fs::read(state(&case, &program)).unwrap(), before);
}
