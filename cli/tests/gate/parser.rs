use std::collections::BTreeSet;

use serde_json::Value;
use tolearn_core::{program, roadmap, stage, topic};

use crate::repo::read;
use crate::schema::paths::{described_fields, objects};
use crate::schema::{schema, valid_documents, yaml};

const ROADMAP: &str = "examples/llm-agents-base/roadmap.yaml";
const TOPIC: &str = "examples/llm-agents-base/topics/local-runtime.yaml";

#[test]
fn the_roadmap_parser_reads_every_field_the_schema_describes() {
    every_described_field_is_read("roadmap", &[ROADMAP.to_owned()], &|source| {
        roadmap::parse(source).map(drop).map_err(|e| e.to_string())
    });
}

#[test]
fn the_topic_parser_reads_every_field_the_schema_describes() {
    every_described_field_is_read("topic", &[TOPIC.to_owned()], &|source| {
        topic::parse(source).map(drop).map_err(|e| e.to_string())
    });
}

#[test]
fn the_program_parser_reads_every_field_the_schema_describes() {
    every_described_field_is_read("program", &documents("program"), &|source| {
        program::parse(source).map(drop).map_err(|e| e.to_string())
    });
}

#[test]
fn the_stage_parser_reads_every_field_the_schema_describes() {
    every_described_field_is_read("stage", &documents("stage"), &|source| {
        stage::parse(source).map(drop).map_err(|e| e.to_string())
    });
}

type Parse<'a> = &'a dyn Fn(&str) -> Result<(), String>;

fn documents(kind: &str) -> Vec<String> {
    valid_documents(kind)
        .into_iter()
        .map(|(path, _)| path)
        .collect()
}

fn every_described_field_is_read(name: &str, paths: &[String], parse: Parse<'_>) {
    let documents: Vec<(&String, Value)> = paths
        .iter()
        .map(|path| (path, transported(path, parse)))
        .collect();
    let optional = optional_fields(&schema(name));

    for field in described_fields(&schema(name)) {
        let (path, document) = documents
            .iter()
            .find(|(_, document)| carries(document, &field))
            .unwrap_or_else(|| {
                panic!(
                    "`{field}`: no valid {name} document carries it, so nothing proves it is read"
                )
            });
        let broken = optional.contains(&field);
        let mut spoiled = document.clone();
        spoil(&mut spoiled, &field, &field, broken);

        assert!(
            parse(&serde_json::to_string(&spoiled).unwrap()).is_err(),
            "`{field}` can be {} in {path} and the parse still succeeds, \
             so the parser never reads the field",
            if broken {
                "given the wrong type"
            } else {
                "dropped"
            }
        );
    }
}

fn transported(path: &str, parse: Parse<'_>) -> Value {
    let source = read(path);
    parse(&source).unwrap_or_else(|e| panic!("{path}: {e}"));
    let document = yaml::load(&source, path);
    let json = serde_json::to_string(&document).unwrap();
    parse(&json).unwrap_or_else(|e| {
        panic!(
            "{path} stops parsing after the round trip through JSON, \
             so spoiling a field proves nothing: {e}"
        )
    });
    document
}

fn optional_fields(schema: &Value) -> BTreeSet<String> {
    let mut required = BTreeSet::new();
    for (path, node) in objects(schema) {
        let names = node.get("required").and_then(Value::as_array);
        for name in names.into_iter().flatten().filter_map(Value::as_str) {
            required.insert(joined(&path, name));
        }
    }
    described_fields(schema)
        .difference(&required)
        .cloned()
        .collect()
}

fn joined(path: &str, name: &str) -> String {
    if path.is_empty() {
        name.to_owned()
    } else {
        format!("{path}.{name}")
    }
}

fn carries(value: &Value, path: &str) -> bool {
    match path.split_once('.') {
        None => value.get(path).is_some(),
        Some((segment, rest)) => match segment.strip_suffix("[]") {
            Some(name) => value
                .get(name)
                .and_then(Value::as_array)
                .is_some_and(|items| items.iter().any(|item| carries(item, rest))),
            None => value.get(segment).is_some_and(|child| carries(child, rest)),
        },
    }
}

fn spoil(value: &mut Value, remaining: &str, path: &str, broken: bool) {
    match remaining.split_once('.') {
        None => {
            let object = value
                .as_object_mut()
                .unwrap_or_else(|| panic!("`{path}`: `{remaining}` sits in something else"));
            if broken {
                object.insert(remaining.to_owned(), Value::Bool(true));
            } else {
                assert!(
                    object.remove(remaining).is_some(),
                    "`{path}`: the document does not carry the field"
                );
            }
        }
        Some((segment, rest)) => {
            let child = match segment.strip_suffix("[]") {
                Some(name) => value
                    .get_mut(name)
                    .and_then(Value::as_array_mut)
                    .and_then(|items| items.iter_mut().find(|item| carries(item, rest))),
                None => value.get_mut(segment),
            };
            let child =
                child.unwrap_or_else(|| panic!("`{path}`: the document has no `{segment}`"));
            spoil(child, rest, path, broken);
        }
    }
}
