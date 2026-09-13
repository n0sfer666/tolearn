#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "activity gate: a panic here is the report"
)]

mod support;

use support::states::PROGRAM;
use tolearn_core::state::{Attempt, Pass, Passed, Sitting, StageState, State, key};

fn stage(opened: Option<&str>, passed: Option<&str>, attempts: &[&str]) -> StageState {
    StageState {
        opened: opened.map(str::to_owned),
        passed: passed.map(|on| Passed {
            on: on.to_owned(),
            by: Pass::Exam,
        }),
        attempts: attempts
            .iter()
            .map(|on| Attempt {
                on: (*on).to_owned(),
                by: Sitting::Written,
                model: None,
                per_question: Vec::new(),
            })
            .collect(),
        ..StageState::default()
    }
}

fn active(stages: Vec<(&str, StageState)>) -> Option<String> {
    let mut state = State::new(PROGRAM);
    for (id, entry) in stages {
        state.stages.insert(key(PROGRAM, id), entry);
    }
    state.active()
}

#[test]
fn a_state_without_dates_has_no_last_activity() {
    assert_eq!(active(Vec::new()), None);
    let ticked = StageState {
        ticks: vec!["builds".to_owned()],
        ..StageState::default()
    };
    assert_eq!(active(vec![("voices", ticked)]), None);
}

#[test]
fn the_last_activity_is_the_latest_opening_pass_or_attempt_of_any_stage() {
    assert_eq!(
        active(vec![
            ("voices", stage(Some("2026-09-07"), Some("2026-09-03"), &[])),
            ("envelope", stage(None, None, &["2026-09-05"])),
        ]),
        Some("2026-09-07".to_owned())
    );
    assert_eq!(
        active(vec![
            (
                "voices",
                stage(Some("2026-09-01"), Some("2026-09-08"), &["2026-09-02"])
            ),
            ("envelope", stage(Some("2026-09-06"), None, &[])),
        ]),
        Some("2026-09-08".to_owned())
    );
    assert_eq!(
        active(vec![
            ("voices", stage(Some("2026-09-03"), Some("2026-09-03"), &[])),
            (
                "envelope",
                stage(Some("2026-09-02"), None, &["2026-09-09", "2026-09-04"])
            ),
        ]),
        Some("2026-09-09".to_owned())
    );
}
