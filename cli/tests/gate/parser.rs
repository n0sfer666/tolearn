use serde_json::Value;
use tolearn_core::{roadmap, topic};

use crate::repo::read;
use crate::schema::paths::described_fields;
use crate::schema::{schema, yaml};

const ROADMAP: &str = "examples/llm-agents-base/roadmap.yaml";
const TOPIC: &str = "examples/llm-agents-base/topics/local-runtime.yaml";

#[test]
fn the_roadmap_parser_reads_every_field_the_schema_describes() {
    every_described_field_is_read("roadmap", ROADMAP, &|source| {
        roadmap::parse(source).map(drop).map_err(|e| e.to_string())
    });
}

#[test]
fn the_topic_parser_reads_every_field_the_schema_describes() {
    every_described_field_is_read("topic", TOPIC, &|source| {
        topic::parse(source).map(drop).map_err(|e| e.to_string())
    });
}

type Parse<'a> = &'a dyn Fn(&str) -> Result<(), String>;

fn every_described_field_is_read(name: &str, path: &str, parse: Parse<'_>) {
    let source = read(path);
    parse(&source).unwrap_or_else(|e| panic!("{path}: {e}"));
    let reference = yaml::load(&source, path);
    let transported = serde_json::to_string(&reference).unwrap();
    parse(&transported).unwrap_or_else(|e| {
        panic!(
            "{path} stops parsing after the round trip through JSON, \
             so dropping a field proves nothing: {e}"
        )
    });

    for field in described_fields(&schema(name)) {
        let mut pruned = reference.clone();
        prune(&mut pruned, &field, &field);

        assert!(
            parse(&serde_json::to_string(&pruned).unwrap()).is_err(),
            "`{field}` can be dropped from {path} and the parse still succeeds, \
             so the parser never reads the field"
        );
    }
}

fn prune(value: &mut Value, remaining: &str, path: &str) {
    match remaining.split_once('.') {
        None => {
            let object = value
                .as_object_mut()
                .unwrap_or_else(|| panic!("`{path}`: `{remaining}` sits in something else"));
            assert!(
                object.remove(remaining).is_some(),
                "`{path}`: the reference does not carry the field"
            );
        }
        Some((segment, rest)) => {
            let child = match segment.strip_suffix("[]") {
                Some(name) => value.get_mut(name).and_then(|list| list.get_mut(0)),
                None => value.get_mut(segment),
            };
            let child =
                child.unwrap_or_else(|| panic!("`{path}`: the reference has no `{segment}`"));
            prune(child, rest, path);
        }
    }
}
