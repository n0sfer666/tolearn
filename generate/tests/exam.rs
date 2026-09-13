#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "exam gate: a panic here is the report"
)]

#[allow(dead_code, reason = "the plan helpers are shared with the plan tests")]
mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use support::{Scripted, Up};
use tolearn_core::stage::{Stage, parse};
use tolearn_core::state::Grade;
use tolearn_generate::exam::{self, Paper};
use tolearn_generate::ledger::{self, Record};
use tolearn_generate::{GenerateError, local, online};

const PROGRAM: &str = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";
const NODE: &str = "9d1e7b40-2c6a-4f8e-b3d5-71a0c4e2f9b6";
const AT: i64 = 1_789_000_000;

const VERDICT: &str = r#"Разбор по вопросам.

```json
{"stage": "voices", "per_question": [
  {"id": "q1", "result": "ok"},
  {"id": "q2", "result": "partial", "missed": ["спектр у волн один"]},
  {"id": "q3", "result": "miss", "missed": ["весь ответ"]},
  {"id": "q4", "result": "miss", "missed": ["весь ответ"]}
]}
```"#;

fn stage() -> Stage {
    parse(&std::fs::read_to_string("../examples/chiptune/stages/voices.yaml").unwrap()).unwrap()
}

fn answers() -> BTreeMap<String, String> {
    [
        ("q1", "Пять каналов: два пульса, треугольник, шум и DPCM."),
        ("q2", "Потому что звучат похоже"),
    ]
    .into_iter()
    .map(|(id, text)| (id.to_owned(), text.to_owned()))
    .collect()
}

fn paper<'a>(stage: &'a Stage, answers: &'a BTreeMap<String, String>) -> Paper<'a> {
    Paper {
        program: PROGRAM,
        node: NODE,
        stage,
        level: "Нот не знаю, трекер не открывал",
        locale: "ru",
        answers,
    }
}

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-exam-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn written(data: &Path) -> Vec<Record> {
    ledger::read(&ledger::path(data, PROGRAM)).unwrap()
}

#[test]
fn промпт_несёт_эталоны_ответы_уровень_и_локаль() {
    let stage = stage();
    let answers = answers();
    let prompt = exam::prompt(&paper(&stage, &answers));

    for question in &stage.questions {
        assert!(
            prompt.contains(&question.text),
            "нет вопроса {}",
            question.id
        );
        assert!(
            prompt.contains(&question.answer),
            "нет эталона {}",
            question.id
        );
    }
    assert!(prompt.contains("Пять каналов: два пульса, треугольник, шум и DPCM."));
    assert!(prompt.contains("Потому что звучат похоже"));
    assert_eq!(prompt.matches("нет ответа").count(), 2);
    assert!(prompt.contains("Нот не знаю, трекер не открывал"));
    assert!(prompt.contains("Язык программы: ru"));
    assert!(prompt.contains(r#""stage": "voices""#));
    assert!(prompt.contains("per_question"));
}

#[test]
fn годный_вердикт_сдаётся_одним_вызовом_и_пишется_в_журнал() {
    let data = scratch("sat");
    let stage = stage();
    let answers = answers();
    let model = Scripted::new(vec![VERDICT.to_owned()]);
    let online = online(&Up, &model).unwrap();

    let sat = exam::sit(&online, &data, &paper(&stage, &answers), AT).unwrap();

    let grades: Vec<Grade> = sat.per_question.iter().map(|row| row.result).collect();
    assert_eq!(
        grades,
        [Grade::Ok, Grade::Partial, Grade::Miss, Grade::Miss]
    );
    assert_eq!(sat.per_question[1].missed, ["спектр у волн один"]);
    assert_eq!(model.prompts().len(), 1);
    let records = written(&data);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].step, "exam");
    assert_eq!(records[0].round, None);
    assert_eq!(records[0].program.as_deref(), Some(NODE));
    assert_eq!(records[0].stage.as_deref(), Some("voices"));
    assert_eq!(records[0].at, Some(AT));
}

#[test]
fn ответ_без_вердикта_чинится_один_раз() {
    let data = scratch("mended");
    let stage = stage();
    let answers = answers();
    let model = Scripted::new(vec!["Всё отлично, молодец!".to_owned(), VERDICT.to_owned()]);

    let sat = exam::sit(&local(&model), &data, &paper(&stage, &answers), AT).unwrap();

    assert_eq!(sat.per_question.len(), 4);
    let prompts = model.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(prompts[1].starts_with(&prompts[0]));
    assert!(prompts[1].contains("Всё отлично, молодец!"));
    assert!(prompts[1].contains("в тексте нет JSON-блока с вердиктом"));
    let rounds: Vec<Option<usize>> = written(&data).iter().map(|record| record.round).collect();
    assert_eq!(rounds, [None, Some(1)]);
}

#[test]
fn второй_ответ_без_вердикта_отказывает_с_причиной() {
    let data = scratch("refused");
    let stage = stage();
    let answers = answers();
    let wrong = VERDICT.replace(r#""stage": "voices""#, r#""stage": "tracker""#);
    let model = Scripted::new(vec![wrong]);

    let refused = exam::sit(&local(&model), &data, &paper(&stage, &answers), AT).unwrap_err();

    assert_eq!(refused.code(), "generate.verdict");
    assert!(matches!(refused, GenerateError::Verdict(_)));
    assert!(refused.to_string().contains("вставлен не в тот этап"));
    assert_eq!(model.prompts().len(), 2);
    assert_eq!(written(&data).len(), 2);
}
