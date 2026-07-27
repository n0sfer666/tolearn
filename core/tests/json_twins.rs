#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::roadmap::parse as roadmap;
use tolearn_core::topic::parse as topic;
use tolearn_core::yaml::ParseFailure;

use support::{read, root};

const BUNDLE: &str = "examples/llm-agents-base";

fn twins() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root().join(BUNDLE).join("topics"))
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            (path.extension()? == "yaml").then(|| path.file_stem()?.to_str().map(str::to_owned))?
        })
        .collect();
    names.sort();
    names
}

#[test]
fn the_reference_roadmap_reads_the_same_from_json() {
    let from_yaml = roadmap(&read(&format!("{BUNDLE}/roadmap.yaml"))).unwrap();
    let from_json = roadmap(&read(&format!("{BUNDLE}/roadmap.json"))).unwrap();
    assert_eq!(from_yaml, from_json);
}

#[test]
fn every_reference_topic_reads_the_same_from_json() {
    let names = twins();
    assert!(names.len() >= 7, "the twin corpus shrank: {names:?}");
    for name in names {
        let from_yaml = topic(&read(&format!("{BUNDLE}/topics/{name}.yaml"))).unwrap();
        let from_json = topic(&read(&format!("{BUNDLE}/topics/{name}.json"))).unwrap();
        assert_eq!(from_yaml, from_json, "{name} reads differently from json");
    }
}

#[test]
fn a_broken_json_document_still_reports_where_it_broke() {
    let error =
        roadmap("{\n  \"schema\": \"learning-roadmap/v1\"\n  \"id\": \"x\"\n}\n").unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Syntax);
    assert_eq!(error.line(), 3);
}

#[test]
fn a_json_document_missing_a_field_names_it_like_any_other() {
    let error = roadmap(r#"{"schema": "learning-roadmap/v1"}"#).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::MissingField);
    assert!(error.message().contains("id"), "{error}");
}

#[test]
fn a_json_value_left_empty_is_read_as_null_and_not_as_a_syntax_error() {
    let error = roadmap(r#"{"schema": "learning-roadmap/v1", "id": }"#).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.path(), "id");
}
