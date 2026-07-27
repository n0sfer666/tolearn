#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integrity gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeSet;

use tolearn_core::bundle::{Violation, validate};

use support::bundles::{document, reference};

fn every_violation() -> Vec<Violation> {
    vec![
        Violation::EmptyStages,
        Violation::EmptyTopics,
        Violation::DuplicateTopicId { id: "a".to_owned() },
        Violation::StageOutOfRange {
            topic: "a".to_owned(),
            stage: 9,
        },
        Violation::UnknownCheckpoint {
            stage: 1,
            checkpoint: "a".to_owned(),
        },
        Violation::CheckpointOutsideStage {
            stage: 1,
            checkpoint: "a".to_owned(),
            found: 2,
        },
        Violation::MissingTopicFile {
            topic: "a".to_owned(),
            stage: 1,
            file: "topics/a.yaml".to_owned(),
        },
        Violation::SchemaMajorMismatch {
            topic: "a".to_owned(),
            roadmap: 1,
            found: 2,
        },
        Violation::UnknownDependency {
            topic: "a".to_owned(),
            depends_on: "b".to_owned(),
        },
        Violation::DependencyCycle {
            chain: vec!["a".to_owned(), "b".to_owned()],
        },
    ]
}

#[test]
fn every_violation_carries_its_own_machine_code() {
    let all = every_violation();
    let codes: BTreeSet<&str> = all.iter().map(Violation::code).collect();
    assert_eq!(
        codes.len(),
        all.len(),
        "two violations share a code: {codes:?}"
    );
}

#[test]
fn every_code_belongs_to_the_bundle_dictionary() {
    for violation in every_violation() {
        let code = violation.code();
        assert!(code.starts_with("bundle."), "{code} is not a bundle code");
    }
}

#[test]
fn every_violation_says_more_for_a_human_than_its_code() {
    for violation in every_violation() {
        let text = violation.to_string();
        let code = violation.code();
        assert!(text.starts_with(code), "{text} does not open with {code}");
        assert!(
            text.len() > code.len() + 2,
            "{code} has nothing to say beyond itself"
        );
    }
}

#[test]
fn a_cycle_closes_on_itself_when_it_is_written_out() {
    let cycle = Violation::DependencyCycle {
        chain: vec!["a".to_owned(), "b".to_owned()],
    };
    assert!(
        cycle.to_string().ends_with("a -> b -> a"),
        "the chain does not close: {cycle}"
    );
}

#[test]
fn a_schema_that_carries_no_version_cannot_match_the_program() {
    let (map, mut topics) = reference();
    let index = document(&topics, "structured-output");
    topics[index].schema = "learning-roadmap/topic".to_owned();

    assert_eq!(
        validate(&map, &topics),
        [Violation::SchemaMajorMismatch {
            topic: "structured-output".to_owned(),
            roadmap: 1,
            found: 0,
        }]
    );
}
