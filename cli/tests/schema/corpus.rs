use serde_json::Value;

use super::{programs, yaml};
use crate::repo::{read, root};

pub fn document(path: &str) -> Value {
    yaml::load(&read(path), path)
}

pub fn fixtures(set: &str, kind: &str) -> Vec<String> {
    listed(&format!("fixtures/v2/{set}/{kind}"))
}

pub fn listed(relative: &str) -> Vec<String> {
    if root().join(relative).is_dir() {
        files(relative, ".yaml")
    } else {
        Vec::new()
    }
}

pub fn valid_documents(kind: &str) -> Vec<(String, Value)> {
    programs::documents(kind)
        .into_iter()
        .map(|path| {
            let document = document(&path);
            (path, document)
        })
        .collect()
}

fn files(relative: &str, extension: &str) -> Vec<String> {
    let dir = root().join(relative);
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    let mut found: Vec<String> = entries
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(extension))
        .map(|name| format!("{relative}/{name}"))
        .collect();
    assert!(!found.is_empty(), "{relative}: no *{extension} files");
    found.sort();
    found
}
