#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeMap;
use std::path::Path;

use saphyr::{LoadableYamlNode, Scalar, Yaml};
use tolearn_core::block::{PREFIX, id, ids};

const SOUND: &str = "Звук — это колебание.";
const TOUCHED: &str = "Звук — это колебание!";

#[test]
fn один_текст_даёт_запиненный_id() {
    assert_eq!(id(SOUND), "8cac2a0a");
    assert_eq!(id("chiptune"), "f28235f6");
    assert_eq!(id(SOUND).len(), PREFIX);
}

#[test]
fn изменённый_символ_даёт_другой_id() {
    assert_eq!(id(TOUCHED), "b2a6877a");
    assert_ne!(id(SOUND), id(TOUCHED));
}

#[test]
fn повтор_в_этапе_получает_суффикс_по_порядку_появления() {
    let stage = ids([SOUND, "chiptune", SOUND, SOUND]);

    assert_eq!(stage, ["8cac2a0a", "f28235f6", "8cac2a0a-2", "8cac2a0a-3"]);
}

#[test]
fn перестановка_несовпадающих_блоков_их_id_не_меняет() {
    let straight = ids(["один", "два", "три"]);
    let turned = ids(["три", "один", "два"]);

    assert_eq!(
        turned,
        [
            straight[2].clone(),
            straight[0].clone(),
            straight[1].clone()
        ]
    );
}

#[test]
fn на_корпусе_фикстур_и_документации_префиксы_не_совпадают() {
    let fixtures = corpus(&["examples", "fixtures"]);
    let texts = corpus(&["examples", "fixtures", "docs"]);

    let mut owners: BTreeMap<String, &str> = BTreeMap::new();
    for text in &texts {
        if let Some(other) = owners.insert(id(text), text) {
            panic!("`{text}` и `{other}` делят id {}", id(text));
        }
    }
    assert!(
        fixtures.len() > 150,
        "корпус фикстур подозрительно мал: {}",
        fixtures.len()
    );
    assert!(
        texts.len() > 1000,
        "корпус подозрительно мал: {}",
        texts.len()
    );
}

fn corpus(rooms: &[&str]) -> Vec<String> {
    let mut texts = Vec::new();
    for room in rooms {
        collect(&support::root().join(room), &mut texts);
    }
    texts.sort();
    texts.dedup();
    texts
}

fn collect(room: &Path, texts: &mut Vec<String>) {
    for entry in std::fs::read_dir(room).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, texts);
            continue;
        }
        let extension = path.extension().and_then(|extension| extension.to_str());
        if !matches!(extension, Some("yaml" | "json" | "md")) {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        if extension == Some("md") {
            texts.extend(
                source
                    .split("\n\n")
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .map(str::to_owned),
            );
            continue;
        }
        let Ok(documents) = Yaml::load_from_str(&source) else {
            continue;
        };
        for document in &documents {
            strings(document, texts);
        }
    }
}

fn strings(node: &Yaml<'_>, texts: &mut Vec<String>) {
    match node {
        Yaml::Value(Scalar::String(text)) => texts.push(text.to_string()),
        Yaml::Sequence(items) => items.iter().for_each(|item| strings(item, texts)),
        Yaml::Mapping(map) => map.values().for_each(|value| strings(value, texts)),
        _ => {}
    }
}
