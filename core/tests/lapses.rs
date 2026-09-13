#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "lapses gate: a panic here is the report"
)]

mod support;

use support::states::PROGRAM;
use tolearn_core::program::{self, Tree};
use tolearn_core::state::{Answered, Attempt, Grade, Lapse, Sitting, State, key};

const STRANGER: &str = "0b1c2d3e-4f50-4a61-8b72-9c83d4e5f607";

fn chiptune() -> Tree {
    program::load(&support::root().join("examples/chiptune")).unwrap()
}

fn row(id: &str, result: Grade, missed: &[&str]) -> Answered {
    Answered {
        id: id.to_owned(),
        result,
        missed: missed.iter().map(|&line| line.to_owned()).collect(),
    }
}

fn sat(rows: Vec<Answered>) -> Attempt {
    Attempt {
        on: "2026-09-14".to_owned(),
        by: Sitting::Written,
        model: None,
        per_question: rows,
    }
}

fn lapse(tree: &Tree, id: &str, missed: &[&str]) -> Lapse {
    let voices = &tree.stages["voices"];
    let question = voices.questions.iter().find(|question| question.id == id);
    Lapse {
        stage: voices.title.clone(),
        question: question.unwrap().text.clone(),
        missed: missed.iter().map(|&line| line.to_owned()).collect(),
    }
}

#[test]
fn незачтённое_берётся_из_последней_попытки_с_текстом_вопроса_и_упущенным() {
    let tree = chiptune();
    let mut state = State::new(PROGRAM);
    state.attempt(
        PROGRAM,
        "voices",
        sat(vec![
            row("q1", Grade::Miss, &["всё"]),
            row("q2", Grade::Miss, &["спектр"]),
        ]),
    );
    state.attempt(
        PROGRAM,
        "voices",
        sat(vec![
            row("q1", Grade::Partial, &["канал DPCM", "шум"]),
            row("q2", Grade::Ok, &[]),
            row("q3", Grade::Miss, &[]),
        ]),
    );

    assert_eq!(
        state.lapses(&tree),
        [
            lapse(&tree, "q1", &["канал DPCM", "шум"]),
            lapse(&tree, "q3", &[]),
        ]
    );
}

#[test]
fn сданная_последняя_попытка_пропуск_и_пустое_состояние_незачтённого_не_дают() {
    let tree = chiptune();
    let mut passed = State::new(PROGRAM);
    passed.attempt(
        PROGRAM,
        "voices",
        sat(vec![row("q1", Grade::Miss, &["всё"])]),
    );
    passed.attempt(PROGRAM, "voices", sat(vec![row("q1", Grade::Ok, &[])]));
    let mut skipped = State::new(PROGRAM);
    skipped.skip(PROGRAM, "voices", "2026-09-14");

    assert!(passed.lapses(&tree).is_empty());
    assert!(skipped.lapses(&tree).is_empty());
    assert!(State::new(PROGRAM).lapses(&tree).is_empty());
}

#[test]
fn этапы_вне_дерева_и_неизвестные_вопросы_пропускаются() {
    let tree = chiptune();
    let mut state = State::new(PROGRAM);
    let failed = || sat(vec![row("q1", Grade::Miss, &["всё"])]);
    state.attempt(PROGRAM, "envelope", failed());
    state.attempt(PROGRAM, "gone", failed());
    state.attempt(STRANGER, "voices", failed());
    state.attempt(
        PROGRAM,
        "voices",
        sat(vec![row("q9", Grade::Miss, &["всё"])]),
    );

    assert!(state.lapses(&tree).is_empty());
}

#[test]
fn перезапись_прячет_прежние_попытки_и_чистит_ответы_и_галочки() {
    let tree = chiptune();
    let mut state = State::new(PROGRAM);
    state.attempt(
        PROGRAM,
        "voices",
        sat(vec![row("q1", Grade::Miss, &["всё"])]),
    );
    let entry = state.stages.get_mut(&key(PROGRAM, "voices")).unwrap();
    entry.ticks = vec!["c1".to_owned()];
    entry.answers.insert("q1".to_owned(), "Пять.".to_owned());
    let passed = entry.passed.clone();

    state.rewritten(PROGRAM, "voices");

    let entry = &state.stages[&key(PROGRAM, "voices")];
    assert!(entry.ticks.is_empty());
    assert!(entry.answers.is_empty());
    assert_eq!(entry.attempts.len(), 1);
    assert_eq!(entry.since, 1);
    assert_eq!(entry.passed, passed);
    assert!(state.last_attempt(PROGRAM, "voices").is_none());
    assert!(state.lapses(&tree).is_empty());

    state.attempt(
        PROGRAM,
        "voices",
        sat(vec![row("q4", Grade::Partial, &["октава"])]),
    );

    assert_eq!(
        state.last_attempt(PROGRAM, "voices").unwrap().on,
        "2026-09-14"
    );
    assert_eq!(state.lapses(&tree), [lapse(&tree, "q4", &["октава"])]);
}

#[test]
fn перезапись_незнакомого_этапа_ничего_не_заводит() {
    let mut state = State::new(PROGRAM);

    state.rewritten(PROGRAM, "voices");

    assert_eq!(state, State::new(PROGRAM));
}
