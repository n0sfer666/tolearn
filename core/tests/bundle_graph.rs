#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integrity gate: a panic here is the report"
)]

mod support;

use tolearn_core::bundle::{Violation, validate};

use support::bundles::{corpus, document, reference};

#[test]
fn a_dependency_on_a_topic_that_does_not_exist_is_reported() {
    let (map, mut topics) = reference();
    let index = document(&topics, "model-selection");
    topics[index].depends_on.push("no-such-topic".to_owned());

    assert_eq!(
        validate(&map, &topics),
        [Violation::UnknownDependency {
            topic: "model-selection".to_owned(),
            depends_on: "no-such-topic".to_owned(),
        }]
    );
}

#[test]
fn a_cycle_is_reported_with_the_chain_that_closes_it() {
    let (map, topics) = corpus();

    assert_eq!(
        validate(&map, &topics),
        [Violation::DependencyCycle {
            chain: vec!["cycle-a".to_owned(), "cycle-b".to_owned()],
        }]
    );
}

#[test]
fn a_longer_cycle_keeps_every_link_in_the_chain() {
    let (map, mut topics) = reference();
    let index = document(&topics, "local-runtime");
    topics[index].depends_on.push("provider-routing".to_owned());

    assert_eq!(
        validate(&map, &topics),
        [Violation::DependencyCycle {
            chain: vec![
                "local-runtime".to_owned(),
                "provider-routing".to_owned(),
                "openai-compatible-api".to_owned(),
            ],
        }]
    );
}

#[test]
fn a_topic_that_depends_on_itself_is_a_cycle_of_one() {
    let (map, mut topics) = reference();
    let index = document(&topics, "provider-routing");
    topics[index].depends_on.push("provider-routing".to_owned());

    assert_eq!(
        validate(&map, &topics),
        [Violation::DependencyCycle {
            chain: vec!["provider-routing".to_owned()],
        }]
    );
}

#[test]
fn one_cycle_is_reported_once_however_many_of_its_topics_start_the_walk() {
    let (map, mut topics) = reference();
    let index = document(&topics, "local-runtime");
    topics[index].depends_on.push("model-selection".to_owned());

    let found = validate(&map, &topics);
    assert_eq!(
        found.len(),
        1,
        "the same cycle was counted twice: {found:?}"
    );
}
