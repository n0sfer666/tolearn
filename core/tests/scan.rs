#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "scanner gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use tolearn_core::bundle::validate;
use tolearn_core::progress::Format;
use tolearn_core::scan::{Scan, ScanError, scan};

use support::{read, root};

const REFERENCE: &str = "examples/llm-agents-base";
const REFERENCE_TOPICS: [&str; 7] = [
    "local-runtime",
    "openai-compatible-api",
    "tokens-context-cost",
    "structured-output",
    "model-selection",
    "provider-routing",
    "cp-gateway",
];

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-scan-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(directory.join("topics")).unwrap();
    directory
}

fn put(root: &Path, name: &str, text: &str) {
    std::fs::write(root.join(name), text).unwrap();
}

fn bundle(name: &str, format: Format) -> PathBuf {
    let root = scratch(name);
    let manifest = match format {
        Format::Yaml => "roadmap.yaml",
        Format::Json => "roadmap.json",
    };
    put(&root, manifest, &read(&format!("{REFERENCE}/{manifest}")));
    for topic in REFERENCE_TOPICS {
        put(
            &root,
            &format!("topics/{topic}.yaml"),
            &read(&format!("{REFERENCE}/topics/{topic}.yaml")),
        );
    }
    root
}

fn done(root: &Path) -> Scan {
    scan(root).unwrap_or_else(|error| panic!("the bundle is expected to open: {error}"))
}

fn ids(scan: &Scan) -> Vec<&str> {
    scan.topics.iter().map(|topic| topic.id.as_str()).collect()
}

#[test]
fn the_root_is_the_directory_that_holds_the_roadmap() {
    let root = bundle("yaml-root", Format::Yaml);

    let found = done(&root);

    assert_eq!(found.root, root);
    assert_eq!(found.roadmap.id, "llm-agents-base");
}

#[test]
fn the_format_of_the_bundle_is_the_one_its_roadmap_is_written_in() {
    assert_eq!(
        done(&bundle("yaml-format", Format::Yaml)).format,
        Format::Yaml
    );
    assert_eq!(
        done(&bundle("json-format", Format::Json)).format,
        Format::Json
    );
}

#[test]
fn the_same_bundle_reads_the_same_whichever_format_the_roadmap_is_in() {
    let from_yaml = done(&bundle("yaml-equal", Format::Yaml));
    let from_json = done(&bundle("json-equal", Format::Json));

    assert_eq!(from_yaml.roadmap, from_json.roadmap);
    assert_eq!(from_yaml.topics, from_json.topics);
}

#[test]
fn two_roadmaps_side_by_side_are_refused_instead_of_guessed() {
    let root = bundle("both", Format::Yaml);
    put(
        &root,
        "roadmap.json",
        &read(&format!("{REFERENCE}/roadmap.json")),
    );

    let error = scan(&root).unwrap_err();

    assert!(
        matches!(&error, ScanError::AmbiguousFormat { root: at } if at == &root),
        "{error}"
    );
}

#[test]
fn a_directory_without_a_roadmap_is_no_bundle() {
    let root = scratch("empty");

    let error = scan(&root).unwrap_err();

    assert!(
        matches!(&error, ScanError::NoRoadmap { root: at } if at == &root),
        "{error}"
    );
}

#[test]
fn a_roadmap_that_does_not_parse_stops_the_scan() {
    let root = bundle("bad-roadmap", Format::Yaml);
    put(
        &root,
        "roadmap.yaml",
        "schema: learning-roadmap/v1\nid: 7\n",
    );

    let error = scan(&root).unwrap_err();

    assert!(
        matches!(&error, ScanError::Malformed { path, .. } if path == &root.join("roadmap.yaml")),
        "{error}"
    );
}

#[test]
fn a_roadmap_that_cannot_be_read_stops_the_scan() {
    let root = scratch("unreadable");
    std::fs::create_dir(root.join("roadmap.yaml")).unwrap();

    let error = scan(&root).unwrap_err();

    assert!(
        matches!(&error, ScanError::Unreadable { path, .. } if path == &root.join("roadmap.yaml")),
        "{error}"
    );
}

#[test]
fn a_topic_of_a_stage_nobody_generated_yet_is_absent_not_lost() {
    let found = done(&bundle("absent", Format::Yaml));

    let absent = found
        .absent
        .iter()
        .find(|absent| absent.id == "tool-calling")
        .unwrap_or_else(|| panic!("`tool-calling` is expected to be absent"));
    assert_eq!(absent.stage, 2);
    assert!(!absent.generated);
    assert_eq!(absent.file, "topics/tool-calling.yaml");
}

#[test]
fn every_topic_of_a_stage_nobody_generated_yet_is_reported() {
    let found = done(&bundle("absent-all", Format::Yaml));

    assert_eq!(found.absent.len(), found.roadmap.topics.len() - 7);
    assert!(found.absent.iter().all(|absent| !absent.generated));
}

#[test]
fn a_file_missing_from_a_generated_stage_is_absent_and_tells_the_stage_was_generated() {
    let root = bundle("missing", Format::Yaml);
    std::fs::remove_file(root.join("topics/model-selection.yaml")).unwrap();

    let found = done(&root);

    let absent = found
        .absent
        .iter()
        .find(|absent| absent.id == "model-selection")
        .unwrap_or_else(|| panic!("`model-selection` is expected to be absent"));
    assert!(absent.generated);
    assert!(!ids(&found).contains(&"model-selection"));
}

#[test]
fn a_broken_topic_file_becomes_a_stub_and_the_rest_are_read() {
    let root = bundle("broken", Format::Yaml);
    put(&root, "topics/structured-output.yaml", "id: 7\n");

    let found = done(&root);

    assert_eq!(found.broken.len(), 1);
    assert_eq!(found.broken[0].id, "structured-output");
    assert_eq!(found.broken[0].file, "topics/structured-output.yaml");
    assert_eq!(ids(&found).len(), 6);
    assert!(!ids(&found).contains(&"structured-output"));
}

#[test]
fn a_broken_topic_carries_the_complaint_of_the_parser() {
    let root = bundle("broken-error", Format::Yaml);
    put(&root, "topics/cp-gateway.yaml", "id: 7\n");

    let found = done(&root);

    let complaint = found.broken[0].error.to_string();
    assert!(complaint.contains("schema"), "{complaint}");
}

#[test]
fn the_topics_come_back_in_the_order_of_the_roadmap() {
    let found = done(&bundle("order", Format::Yaml));

    assert_eq!(ids(&found), REFERENCE_TOPICS);
}

#[test]
fn a_topic_the_roadmap_does_not_name_is_not_picked_up_from_the_directory() {
    let root = bundle("stray", Format::Yaml);
    put(
        &root,
        "topics/stray.yaml",
        &read(&format!("{REFERENCE}/topics/cp-gateway.yaml")),
    );

    let found = done(&root);

    assert_eq!(ids(&found).len(), 7);
}

#[test]
fn the_example_carries_both_formats_and_is_a_corpus_rather_than_a_bundle() {
    let error = scan(&root().join(REFERENCE)).unwrap_err();

    assert!(
        matches!(&error, ScanError::AmbiguousFormat { .. }),
        "{error}"
    );
}

#[test]
fn what_the_scanner_hands_over_passes_the_validator() {
    let found = done(&bundle("valid", Format::Yaml));

    assert_eq!(validate(&found.roadmap, &found.topics), []);
}
