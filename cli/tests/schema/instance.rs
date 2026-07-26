use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::{KINDS, valid_documents};

pub type Observed = BTreeMap<String, BTreeSet<String>>;

pub fn strings_by_path(instance: &Value) -> Observed {
    let mut observed = Observed::new();
    visit(instance, String::new(), 0, &mut observed);
    observed
}

pub fn values_at(observed: &Observed, pattern: &str) -> BTreeSet<String> {
    observed
        .iter()
        .filter(|(path, _)| covered_by(path, pattern))
        .flat_map(|(_, values)| values.iter().cloned())
        .collect()
}

pub fn observed_by_kind() -> BTreeMap<&'static str, Observed> {
    KINDS
        .into_iter()
        .map(|kind| {
            let mut merged = Observed::new();
            for (_, document) in valid_documents(kind) {
                for (path, values) in strings_by_path(&document) {
                    merged.entry(path).or_default().extend(values);
                }
            }
            (kind, merged)
        })
        .collect()
}

pub fn attempts(progress: &Value) -> Vec<Value> {
    progress["topics"]
        .as_object()
        .map(|topics| {
            topics
                .values()
                .filter_map(|topic| topic["attempts"].as_array())
                .flatten()
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

pub fn topic_ids(roadmap: &Value) -> Vec<String> {
    roadmap["topics"]
        .as_array()
        .map(|topics| {
            topics
                .iter()
                .filter_map(|topic| topic["id"].as_str())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn visit(node: &Value, path: String, depth: usize, observed: &mut Observed) {
    assert!(depth < 32, "instance nested deeper than 32 levels");
    match node {
        Value::String(text) => {
            observed.entry(path).or_default().insert(text.clone());
        }
        Value::Array(items) => {
            let path = format!("{path}[]");
            for item in items {
                visit(item, path.clone(), depth + 1, observed);
            }
        }
        Value::Object(entries) => {
            for (key, value) in entries {
                assert!(
                    !key.contains('.'),
                    "`{path}`: key `{key}` holds a dot, and paths are joined by dots"
                );
                visit(value, join(&path, key), depth + 1, observed);
            }
        }
        _ => {}
    }
}

fn covered_by(path: &str, pattern: &str) -> bool {
    let mut segments = path.split('.');
    let mut wanted = pattern.split('.');
    loop {
        match (segments.next(), wanted.next()) {
            (None, None) => return true,
            (Some(segment), Some(want)) if want == "*" || want == segment => {}
            _ => return false,
        }
    }
}

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}
