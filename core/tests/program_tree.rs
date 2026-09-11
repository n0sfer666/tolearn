#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "program tree gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;

use tolearn_core::program::{self, Violation, load, validate};
use tolearn_core::stage;

use support::{line_of, read, root};

const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
const NODE: &str = "b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56";
const LEAF: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";
const INVALID: &str = "fixtures/v2/invalid";
const STRICT: [(&str, &str); 6] = [
    ("program/hours-reversed.yaml", "program.hours-reversed"),
    (
        "program/stages-and-children.yaml",
        "program.node-with-stages",
    ),
    ("stage/block-id-not-from-text.yaml", "program.block-id"),
    (
        "stage/diagram-without-asset.yaml",
        "program.block-without-asset",
    ),
    (
        "stage/image-without-license.yaml",
        "program.unlicensed-image",
    ),
    (
        "stage/lang-on-a-paragraph.yaml",
        "program.foreign-block-field",
    ),
];

#[test]
fn the_reference_loads_whole_and_breaks_no_rule() {
    let tree = load(&root().join("examples/chiptune")).unwrap();

    assert_eq!(validate(&tree), []);
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["voices"]);
    assert!(
        tree.assets.contains("assets/voices.svg"),
        "{:?}",
        tree.assets
    );
    assert!(
        tree.assets.contains("assets/pulse-wave.png"),
        "{:?}",
        tree.assets
    );
}

#[test]
fn the_valid_tree_loads_three_levels_deep_and_breaks_no_rule() {
    let tree = load(&root().join(NES_DEV)).unwrap();

    assert_eq!(validate(&tree), []);
    let leaf = &tree.children[NODE].children[LEAF];
    assert_eq!(
        leaf.stages.keys().collect::<Vec<_>>(),
        ["first-rom", "linker"]
    );
    assert!(leaf.children.is_empty());
}

#[test]
fn a_child_row_without_a_directory_is_a_subprogram_not_yet_generated() {
    let tree = load(&root().join(NES_DEV)).unwrap();

    assert_eq!(tree.program.map.children.len(), 2);
    assert_eq!(tree.children.keys().collect::<Vec<_>>(), [NODE]);
}

#[test]
fn every_invalid_tree_is_refused_with_the_code_in_its_name_and_nothing_else() {
    let mut seen = 0;
    for directory in directories(INVALID) {
        let name = directory
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let (code, _) = name
            .split_once("__")
            .unwrap_or_else(|| panic!("{name}: the name carries no `<code>__`"));
        let observed: Vec<String> = match load(&directory) {
            Err(error) => vec![error.code().to_owned()],
            Ok(tree) => validate(&tree).iter().map(ToString::to_string).collect(),
        };
        assert_eq!(observed.len(), 1, "{name}: {observed:?}");
        assert!(observed[0].starts_with(code), "{name}: {observed:?}");
        seen += 1;
    }
    assert!(seen >= 6, "{INVALID} lost its trees: {seen} of 6");
}

#[test]
fn a_stage_that_does_not_parse_is_reported_by_its_file_and_line() {
    let relative = format!("{INVALID}/yaml.wrong-type__stage-title-a-list");
    let stage = format!("{relative}/stages/linker.yaml");

    let error = load(&root().join(&relative)).unwrap_err();

    assert_eq!(error.path(), root().join(&stage));
    assert_eq!(error.line(), Some(line_of(&read(&stage), "title:")));
    assert!(error.to_string().contains("stages/linker.yaml"), "{error}");
}

#[test]
fn every_strict_document_is_refused_by_its_own_rule() {
    for (file, code) in STRICT {
        let source = read(&format!("fixtures/v2/strict/{file}"));
        let found: Vec<Violation> = if file.starts_with("program/") {
            program::check(&program::parse(&source).unwrap())
        } else {
            stage::check(&stage::parse(&source).unwrap())
        };
        let codes: Vec<&str> = found.iter().map(Violation::code).collect();
        assert_eq!(codes, [code], "{file}: {found:?}");
    }
}

#[test]
fn every_strict_document_is_pinned_to_its_rule() {
    for kind in ["program", "stage"] {
        for path in files(&format!("fixtures/v2/strict/{kind}")) {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let file = format!("{kind}/{name}");
            assert!(
                STRICT.iter().any(|(pinned, _)| *pinned == file),
                "fixtures/v2/strict/{file} is pinned to no rule"
            );
        }
    }
}

#[test]
fn a_violation_names_its_code_and_where_it_stands() {
    let source = read("fixtures/v2/strict/program/hours-reversed.yaml");

    let found = program::check(&program::parse(&source).unwrap());

    assert_eq!(
        found[0].to_string(),
        "program.hours-reversed: `voices` is estimated at 3 hours at the least and 2 at the most"
    );
}

fn directories(relative: &str) -> Vec<PathBuf> {
    entries(relative)
        .into_iter()
        .filter(|path| path.is_dir())
        .collect()
}

fn files(relative: &str) -> Vec<PathBuf> {
    entries(relative)
        .into_iter()
        .filter(|path| path.is_file())
        .collect()
}

fn entries(relative: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(root().join(relative))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    found.sort();
    found
}
