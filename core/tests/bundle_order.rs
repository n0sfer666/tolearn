#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integrity gate: a panic here is the report"
)]

mod support;

use tolearn_core::bundle::{Violation, ordered, tracked, validate};
use tolearn_core::roadmap::Roadmap;

use support::bundles::{document, entry, recorded_for, reference};

#[test]
fn a_plan_whose_dependencies_all_point_backwards_is_accepted() {
    let (map, topics) = reference();

    assert_eq!(ordered(&map, &topics), []);
}

#[test]
fn a_dependency_on_a_later_topic_is_reported() {
    let (map, mut topics) = reference();
    let index = document(&topics, "local-runtime");
    topics[index].depends_on.push("model-selection".to_owned());

    assert_eq!(
        ordered(&map, &topics),
        [Violation::ForwardDependency {
            topic: "local-runtime".to_owned(),
            depends_on: "model-selection".to_owned(),
        }]
    );
}

#[test]
fn a_topic_that_depends_on_itself_is_a_forward_dependency() {
    let (map, mut topics) = reference();
    let index = document(&topics, "provider-routing");
    topics[index].depends_on.push("provider-routing".to_owned());

    assert_eq!(
        ordered(&map, &topics),
        [Violation::ForwardDependency {
            topic: "provider-routing".to_owned(),
            depends_on: "provider-routing".to_owned(),
        }]
    );
}

#[test]
fn a_dependency_the_program_does_not_declare_is_left_to_the_whole_bundle_check() {
    let (map, mut topics) = reference();
    let index = document(&topics, "model-selection");
    topics[index].depends_on.push("no-such-topic".to_owned());

    assert_eq!(ordered(&map, &topics), []);
}

#[test]
fn a_pair_that_closes_a_cycle_names_the_edge_that_points_forward() {
    let (map, mut topics) = reference();
    let first = document(&topics, "tokens-context-cost");
    topics[first].depends_on.push("model-selection".to_owned());
    let second = document(&topics, "model-selection");
    topics[second]
        .depends_on
        .push("tokens-context-cost".to_owned());

    assert_eq!(
        ordered(&map, &topics),
        [Violation::ForwardDependency {
            topic: "tokens-context-cost".to_owned(),
            depends_on: "model-selection".to_owned(),
        }]
    );
}

#[test]
fn a_topic_kept_outside_the_program_directory_is_reported() {
    let (mut map, topics) = reference();
    let index = entry(&map, "local-runtime");
    map.topics[index].file = "../topics/local-runtime.yaml".to_owned();

    assert!(
        validate(&map, &topics).contains(&Violation::UnsafeTopicFile {
            topic: "local-runtime".to_owned(),
            file: "../topics/local-runtime.yaml".to_owned(),
        })
    );
}

#[test]
fn an_absolute_topic_file_is_reported() {
    let (mut map, topics) = reference();
    let index = entry(&map, "local-runtime");
    map.topics[index].file = "/etc/tolearn.yaml".to_owned();

    assert!(
        validate(&map, &topics).contains(&Violation::UnsafeTopicFile {
            topic: "local-runtime".to_owned(),
            file: "/etc/tolearn.yaml".to_owned(),
        })
    );
}

#[test]
fn two_topics_that_share_a_document_are_reported_once() {
    let (mut map, topics) = reference();
    let index = entry(&map, "local-runtime");
    let taken = map.topics[entry(&map, "model-selection")].file.clone();
    map.topics[index].file = taken.clone();

    let found: Vec<Violation> = validate(&map, &topics)
        .into_iter()
        .filter(|violation| matches!(violation, Violation::DuplicateTopicFile { .. }))
        .collect();

    assert_eq!(found, [Violation::DuplicateTopicFile { file: taken }]);
}

fn every_topic(map: &Roadmap) -> Vec<(&str, &str)> {
    map.topics
        .iter()
        .map(|entry| (entry.id.as_str(), "todo"))
        .collect()
}

#[test]
fn a_progress_that_lists_every_topic_of_the_program_is_accepted() {
    let (map, _) = reference();
    let progress = recorded_for(&map.id, &every_topic(&map));

    assert_eq!(tracked(&map, &progress), []);
}

#[test]
fn a_progress_written_for_another_program_is_reported() {
    let (map, _) = reference();
    let progress = recorded_for("corpus-program", &every_topic(&map));

    assert!(
        tracked(&map, &progress).contains(&Violation::ProgressForAnotherProgram {
            program: map.id.clone(),
            found: "corpus-program".to_owned(),
        })
    );
}

#[test]
fn a_topic_missing_from_the_progress_is_reported() {
    let (map, _) = reference();
    let mut pairs = every_topic(&map);
    let dropped = pairs.remove(0).0.to_owned();
    let progress = recorded_for(&map.id, &pairs);

    assert_eq!(
        tracked(&map, &progress),
        [Violation::UntrackedTopic { topic: dropped }]
    );
}

#[test]
fn a_progress_entry_the_program_does_not_declare_is_reported() {
    let (map, _) = reference();
    let mut pairs = every_topic(&map);
    pairs.push(("no-such-topic", "todo"));
    let progress = recorded_for(&map.id, &pairs);

    assert_eq!(
        tracked(&map, &progress),
        [Violation::StrayProgressTopic {
            topic: "no-such-topic".to_owned(),
        }]
    );
}
