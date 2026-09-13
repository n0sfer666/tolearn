#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "attempts gate: a panic here is the report"
)]

mod support;

use support::states::{PROGRAM, scratch};
use tolearn_core::state::{Answered, Attempt, Grade, Pass, Passed, Sitting, State, Status, key};

fn row(id: &str, result: Grade, missed: &[&str]) -> Answered {
    Answered {
        id: id.to_owned(),
        result,
        missed: missed.iter().map(|&line| line.to_owned()).collect(),
    }
}

fn sat(on: &str, results: &[Grade]) -> Attempt {
    Attempt {
        on: on.to_owned(),
        by: Sitting::Written,
        model: Some("qwen3:8b".to_owned()),
        per_question: results
            .iter()
            .enumerate()
            .map(|(at, &result)| {
                let missed: &[&str] = if result == Grade::Ok {
                    &[]
                } else {
                    &["упущено"]
                };
                row(&format!("q{}", at + 1), result, missed)
            })
            .collect(),
    }
}

fn passed(state: &State) -> Option<Passed> {
    state.stages[&key(PROGRAM, "voices")].passed.clone()
}

#[test]
fn все_ok_проходят_этап_сданным_зачётом_в_день_попытки() {
    let mut state = State::new(PROGRAM);
    state.open(PROGRAM, "voices", "2026-09-10");

    let attempt = sat("2026-09-12", &[Grade::Ok, Grade::Ok]);
    assert!(attempt.passes());
    state.attempt(PROGRAM, "voices", attempt.clone());

    assert_eq!(state.status(PROGRAM, "voices"), Status::Passed(Pass::Exam));
    assert_eq!(
        passed(&state),
        Some(Passed {
            on: "2026-09-12".to_owned(),
            by: Pass::Exam
        })
    );
    assert_eq!(state.last_attempt(PROGRAM, "voices"), Some(&attempt));
}

#[test]
fn незачтённое_оставляет_этап_начатым_и_видно_в_последней_попытке() {
    let mut state = State::new(PROGRAM);

    let attempt = sat("2026-09-12", &[Grade::Ok, Grade::Partial]);
    assert!(!attempt.passes());
    state.attempt(PROGRAM, "voices", attempt);

    assert_eq!(state.status(PROGRAM, "voices"), Status::Opened);
    assert_eq!(
        state.stages[&key(PROGRAM, "voices")].opened.as_deref(),
        Some("2026-09-12")
    );
    let last = state.last_attempt(PROGRAM, "voices").unwrap();
    assert_eq!(last.per_question[1].result, Grade::Partial);
    assert_eq!(last.per_question[1].missed, ["упущено"]);
    assert_eq!(state.last_attempt(PROGRAM, "envelope"), None);
}

#[test]
fn проваленная_попытка_пройденный_этап_назад_не_откатывает() {
    let mut state = State::new(PROGRAM);
    state.attempt(PROGRAM, "voices", sat("2026-09-10", &[Grade::Ok]));

    state.attempt(PROGRAM, "voices", sat("2026-09-12", &[Grade::Miss]));
    state.attempt(PROGRAM, "voices", sat("2026-09-13", &[Grade::Ok]));

    assert_eq!(
        passed(&state),
        Some(Passed {
            on: "2026-09-10".to_owned(),
            by: Pass::Exam
        })
    );
    let attempts = &state.stages[&key(PROGRAM, "voices")].attempts;
    assert_eq!(
        attempts
            .iter()
            .map(|one| one.on.as_str())
            .collect::<Vec<_>>(),
        ["2026-09-10", "2026-09-12", "2026-09-13"]
    );
}

#[test]
fn сданный_зачёт_меняет_пропуск_на_зачёт_а_проваленный_нет() {
    let mut state = State::new(PROGRAM);
    state
        .stages
        .entry(key(PROGRAM, "voices"))
        .or_default()
        .passed = Some(Passed {
        on: "2026-09-01".to_owned(),
        by: Pass::Skip,
    });

    state.attempt(PROGRAM, "voices", sat("2026-09-11", &[Grade::Miss]));
    assert_eq!(state.status(PROGRAM, "voices"), Status::Passed(Pass::Skip));

    state.attempt(PROGRAM, "voices", sat("2026-09-12", &[Grade::Ok]));
    assert_eq!(
        passed(&state),
        Some(Passed {
            on: "2026-09-12".to_owned(),
            by: Pass::Exam
        })
    );
}

#[test]
fn попытка_ложится_в_файл_дословно_со_способом_и_моделью() {
    let data = scratch("attempts");
    let written = sat("2026-09-12", &[Grade::Ok, Grade::Miss]);
    let pasted = Attempt {
        on: "2026-09-13".to_owned(),
        by: Sitting::Copypaste,
        model: None,
        per_question: vec![
            row("q1", Grade::Ok, &["мелочь: «скважность»"]),
            row(
                "q2",
                Grade::Partial,
                &["строка: с двоеточием", "# и решёткой"],
            ),
        ],
    };

    State::update(&data, PROGRAM, |state| {
        state.attempt(PROGRAM, "voices", written.clone());
        state.attempt(PROGRAM, "voices", pasted.clone());
    })
    .unwrap();

    let state = State::read(&data, PROGRAM).unwrap();
    assert_eq!(
        state.stages[&key(PROGRAM, "voices")].attempts,
        [written, pasted]
    );
    assert_eq!(state.status(PROGRAM, "voices"), Status::Opened);
}
