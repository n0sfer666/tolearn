#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integrity gate: a panic here is the report"
)]

mod support;

use tolearn_core::bundle::{Violation, validate};

use support::bundles::{document, entry, reference};

#[test]
fn the_reference_bundle_is_whole() {
    let (map, topics) = reference();
    assert_eq!(validate(&map, &topics), []);
}

#[test]
fn a_repeated_topic_id_is_reported_once_by_id() {
    let (mut map, topics) = reference();
    let duplicate = map.topics[entry(&map, "model-selection")].clone();
    map.topics.push(duplicate);

    assert_eq!(
        validate(&map, &topics),
        [
            Violation::DuplicateTopicId {
                id: "model-selection".to_owned(),
            },
            Violation::DuplicateTopicFile {
                file: "topics/model-selection.yaml".to_owned(),
            }
        ]
    );
}

#[test]
fn a_checkpoint_that_names_no_topic_is_reported() {
    let (mut map, topics) = reference();
    map.stages[0].checkpoint = "no-such-checkpoint".to_owned();

    assert_eq!(
        validate(&map, &topics),
        [Violation::UnknownCheckpoint {
            stage: 1,
            checkpoint: "no-such-checkpoint".to_owned(),
        }]
    );
}

#[test]
fn a_checkpoint_belonging_to_another_stage_does_not_cover_this_one() {
    let (mut map, topics) = reference();
    map.stages[0].checkpoint = "cp-agent-stack".to_owned();

    assert_eq!(
        validate(&map, &topics),
        [Violation::CheckpointOutsideStage {
            stage: 1,
            checkpoint: "cp-agent-stack".to_owned(),
            found: 2,
        }]
    );
}

#[test]
fn a_topic_pointing_at_a_stage_that_does_not_exist_is_reported() {
    let (mut map, topics) = reference();
    let index = entry(&map, "provider-routing");
    map.topics[index].stage = 9;

    assert_eq!(
        validate(&map, &topics),
        [Violation::StageOutOfRange {
            topic: "provider-routing".to_owned(),
            stage: 9,
        }]
    );
}

#[test]
fn a_topic_written_against_another_major_version_is_reported() {
    let (map, mut topics) = reference();
    let index = document(&topics, "structured-output");
    topics[index].schema = "learning-roadmap/topic/v2".to_owned();

    assert_eq!(
        validate(&map, &topics),
        [Violation::SchemaMajorMismatch {
            topic: "structured-output".to_owned(),
            roadmap: 1,
            found: 2,
        }]
    );
}

#[test]
fn a_topic_missing_from_a_generated_stage_is_reported() {
    let (map, mut topics) = reference();
    let index = document(&topics, "provider-routing");
    topics.remove(index);

    assert_eq!(
        validate(&map, &topics),
        [Violation::MissingTopicFile {
            topic: "provider-routing".to_owned(),
            stage: 1,
            file: "topics/provider-routing.yaml".to_owned(),
        }]
    );
}

#[test]
fn a_topic_missing_from_a_stage_that_is_not_generated_yet_is_normal() {
    let (map, topics) = reference();
    assert!(map.stages.iter().any(|stage| !stage.generated));
    assert_eq!(validate(&map, &topics), []);
}

#[test]
fn a_program_without_stages_is_reported() {
    let (mut map, topics) = reference();
    map.stages.clear();

    let found = validate(&map, &topics);
    assert!(
        found.contains(&Violation::EmptyStages),
        "empty stages went unreported: {found:?}"
    );
}

#[test]
fn a_program_without_topics_is_reported() {
    let (mut map, _) = reference();
    map.topics.clear();

    let found = validate(&map, &[]);
    assert!(
        found.contains(&Violation::EmptyTopics),
        "empty topics went unreported: {found:?}"
    );
}

#[test]
fn every_violation_is_collected_in_one_pass() {
    let (mut map, mut topics) = reference();
    map.stages[0].checkpoint = "no-such-checkpoint".to_owned();
    let index = entry(&map, "provider-routing");
    map.topics[index].stage = 9;
    let index = document(&topics, "model-selection");
    topics[index].depends_on.push("no-such-topic".to_owned());

    let found = validate(&map, &topics);
    assert_eq!(found.len(), 3, "one pass stopped early: {found:?}");
}

#[test]
fn hours_that_run_backwards_are_reported() {
    let (mut map, topics) = reference();
    let index = entry(&map, "model-selection");
    map.topics[index].est_hours.min = 9;
    map.topics[index].est_hours.max = 4;

    assert_eq!(
        validate(&map, &topics),
        [Violation::HoursReversed {
            topic: "model-selection".to_owned(),
            min: 9,
            max: 4,
        }]
    );
}

#[test]
fn hours_that_run_backwards_in_the_topic_file_are_reported_too() {
    let (map, mut topics) = reference();
    let index = document(&topics, "local-runtime");
    topics[index].est_hours.min = 5;
    topics[index].est_hours.max = 1;

    assert_eq!(
        validate(&map, &topics),
        [Violation::HoursReversed {
            topic: "local-runtime".to_owned(),
            min: 5,
            max: 1,
        }]
    );
}

#[test]
fn hours_that_name_one_and_the_same_number_are_no_violation() {
    let (mut map, topics) = reference();
    let index = entry(&map, "model-selection");
    map.topics[index].est_hours.min = 4;
    map.topics[index].est_hours.max = 4;

    assert_eq!(validate(&map, &topics), []);
}
