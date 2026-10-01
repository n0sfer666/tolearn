#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeSet;

use serde_json::json;
use support::held::{asked, held, state};
use support::starter::Case;
use tolearn_app::ipc::{Hold, IpcError, call};

const OTHER: &str = "9f1c0d33-4b7a-4e2f-8c15-6a3d9e0b7f42";

fn drawn(case: &Case) -> IpcError {
    call(&case.context, "plan_program", &asked()).unwrap_err()
}

#[test]
fn занятая_генерация_называет_что_её_держит_генерацию_зачёт_или_уточнение() {
    let held = held();
    let during = drawn(&held.case);
    held.release.send(()).unwrap();
    held.drawing.join().unwrap().unwrap();
    let case = held.case;
    let heard = case.model.heard().len();
    let mut refusals = vec![(during, "generation")];

    for (hold, name) in [
        (Hold::Exam(OTHER.to_owned()), "exam"),
        (Hold::Clarify(OTHER.to_owned()), "clarify"),
    ] {
        let claim = case.context.running().claim(hold).unwrap();
        refusals.push((drawn(&case), name));
        let quiet = state(&case);
        drop(claim);
        assert_eq!(quiet["work"], json!(null), "{name}");
        assert!(quiet["outcome"]["plan"].is_object(), "{name}");
    }

    for (refused, name) in &refusals {
        assert_eq!(refused.code, "generate.busy", "{name}");
        assert_eq!(refused.held.as_deref(), Some(*name));
    }
    let messages: BTreeSet<&str> = refusals
        .iter()
        .map(|(refused, _)| refused.message.as_str())
        .collect();
    assert_eq!(messages.len(), 3);
    assert_eq!(case.model.heard().len(), heard);
}
