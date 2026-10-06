use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;
use tolearn_core::block::ids;

use crate::repo::root;
use crate::schema::paths::fixed_values;
use crate::schema::programs::{REFERENCE, children, roots};
use crate::schema::{document, schema, valid_documents};

#[test]
fn every_block_id_in_the_valid_corpus_is_the_s97_id_of_its_text() {
    let stages = valid_documents("stage");
    assert!(!stages.is_empty(), "the v2 valid corpus holds no stage");
    for (path, stage) in stages {
        let blocks = blocks(&stage);
        let stated: Vec<&str> = blocks.iter().map(|block| text(block, "id")).collect();
        let derived = ids(blocks.iter().map(|block| text(block, "text")));
        assert_eq!(
            stated, derived,
            "{path}: block ids are not the S97 ids of their texts"
        );
    }
}

#[test]
fn every_asset_a_valid_block_names_lies_in_its_program() {
    for (path, stage) in valid_documents("stage") {
        let program = Path::new(&path).parent().and_then(Path::parent).unwrap();
        for block in blocks(&stage) {
            if let Some(asset) = block.get("asset").and_then(Value::as_str) {
                let file = root().join(program).join(asset);
                assert!(
                    file.is_file(),
                    "{path}: `{asset}` is missing from {}",
                    program.display()
                );
            }
        }
    }
}

#[test]
fn the_reference_stage_shows_every_block_kind_and_a_practice_without_check() {
    let kinds = &fixed_values(&schema("stage"))["blocks[].kind"];
    let stages: Vec<(String, Value)> = valid_documents("stage")
        .into_iter()
        .filter(|(path, _)| path.starts_with(REFERENCE))
        .collect();
    assert!(!stages.is_empty(), "{REFERENCE} holds no stage");
    for (path, stage) in stages {
        let shown: BTreeSet<String> = stage["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|block| text(block, "kind").to_owned())
            .collect();
        assert_eq!(&shown, kinds, "{path}: not every block kind is shown");
        assert!(
            checks(&stage)
                .iter()
                .all(|check| check.get("check").is_none()),
            "{path}: the reference practice runs nothing, so it carries no `check`"
        );
    }
}

#[test]
fn the_reference_program_cites_a_book_by_chapter_and_a_page() {
    let program = document(&format!("{REFERENCE}/program.yaml"));
    for source in ["books", "pages"] {
        let cited = program["sources"][source].as_array().map_or(0, Vec::len);
        assert!(cited > 0, "{REFERENCE}/program.yaml cites no {source}");
    }
}

#[test]
fn the_valid_corpus_nests_three_levels_and_runs_a_check() {
    let deepest = roots().iter().map(|program| depth(program)).max();
    assert_eq!(
        deepest,
        Some(3),
        "the v2 valid corpus is not nested three levels deep"
    );
    let runs = valid_documents("stage").iter().any(|(_, stage)| {
        checks(stage)
            .iter()
            .any(|check| check.get("check").is_some())
    });
    assert!(runs, "no practice in the v2 valid corpus carries a `check`");
}

fn depth(program: &str) -> usize {
    1 + children(program)
        .iter()
        .map(|child| depth(child))
        .max()
        .unwrap_or(0)
}

fn blocks(stage: &Value) -> Vec<&Value> {
    items(&stage["blocks"])
        .chain(items(&stage["practice"]["task"]))
        .collect()
}

fn checks(stage: &Value) -> Vec<&Value> {
    items(&stage["practice"]["constraints"])
        .chain(items(&stage["practice"]["acceptance"]))
        .collect()
}

fn items(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten()
}

fn text<'a>(block: &'a Value, field: &str) -> &'a str {
    block[field].as_str().unwrap_or_default()
}
