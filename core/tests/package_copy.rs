#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "package copy gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeSet;
use std::fs;

use support::packages::NES_DEV;
use support::programs::{listing, scratch, snapshot, uuid_of};
use support::sealing::{content, packed, sealed, with};
use tolearn_core::library::Library;
use tolearn_core::package::import;
use tolearn_core::program::Tree;

const OUTSIDER: &str = "0f1e2d3c-4b5a-4968-8776-a5b4c3d2e1f0";
const BUILD: &str = "b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56";
const SOUND: &str = "5c8e1f2a-7b3d-4e69-9a04-2d6f8b1c3e75";

fn uuids(tree: &Tree) -> BTreeSet<String> {
    let mut found: BTreeSet<String> = tree.children.values().flat_map(uuids).collect();
    found.insert(tree.program.uuid.clone());
    found.extend(tree.program.map.children.iter().map(|row| row.uuid.clone()));
    found
}

fn everything(library: &Library) -> BTreeSet<String> {
    library
        .list()
        .unwrap()
        .iter()
        .flat_map(|entry| uuids(entry.program.as_ref().unwrap()))
        .collect()
}

fn shape(tree: &Tree) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = tree
        .stages
        .keys()
        .map(|id| (tree.program.title.clone(), id.clone()))
        .chain(tree.children.values().flat_map(shape))
        .collect();
    found.sort();
    found
}

fn rewritten(relative: &str, changes: &[(String, String)]) -> Vec<u8> {
    let files = content(relative);
    let (_, yaml) = files
        .iter()
        .find(|(name, _)| name == "program.yaml")
        .unwrap();
    let mut text = String::from_utf8(yaml.clone()).unwrap();
    for (from, to) in changes {
        assert!(text.contains(from.as_str()), "{from}");
        text = text.replacen(from.as_str(), to, 1);
    }
    sealed(with(files, "program.yaml", text.as_bytes()))
}

#[test]
fn a_taken_uuid_brings_a_copy_with_new_uuids_across_the_tree() {
    let room = scratch("import-copy");
    let data = room.join("data");
    let library = Library::at(&data);
    let file = packed(&room, NES_DEV);
    let original = uuid_of(NES_DEV);
    import(&file, &library).unwrap();
    let before = snapshot(&data.join("programs").join(&original));

    let copied = import(&file, &library).unwrap();

    assert_eq!(copied.copy_of.as_deref(), Some(original.as_str()));
    assert_ne!(copied.uuid, original);
    let copy = library.open(&copied.uuid).unwrap();
    let source = library.open(&original).unwrap();
    assert!(uuids(&copy).is_disjoint(&uuids(&source)));
    assert_eq!(uuids(&copy).len(), uuids(&source).len());
    assert_eq!(copy.program.title, source.program.title);
    assert_eq!(shape(&copy), shape(&source));
    assert_eq!(snapshot(&data.join("programs").join(&original)), before);
    let mut listed = vec![original, copied.uuid];
    listed.sort();
    assert_eq!(listing(&data.join("programs")), listed);
}

#[test]
fn a_subprogram_uuid_already_in_the_library_brings_a_copy_too() {
    let room = scratch("import-copy-child");
    let data = room.join("data");
    let library = Library::at(&data);
    import(&packed(&room, NES_DEV), &library).unwrap();
    let before = everything(&library);
    let file = room.join("renamed.tolearn");
    let root = format!("uuid: {}", uuid_of(NES_DEV));
    fs::write(
        &file,
        rewritten(NES_DEV, &[(root, format!("uuid: {OUTSIDER}"))]),
    )
    .unwrap();

    let copied = import(&file, &library).unwrap();

    assert_eq!(copied.copy_of.as_deref(), Some(OUTSIDER));
    let copy = uuids(&library.open(&copied.uuid).unwrap());
    assert!(copy.is_disjoint(&before), "{copy:?}");
    assert!(!everything(&library).contains(OUTSIDER));
}

#[test]
fn uuids_written_in_any_yaml_form_are_renewed() {
    let room = scratch("import-copy-forms");
    let data = room.join("data");
    let library = Library::at(&data);
    import(&packed(&room, NES_DEV), &library).unwrap();
    let original = uuid_of(NES_DEV);
    let before = uuids(&library.open(&original).unwrap());
    let file = room.join("forms.tolearn");
    let changes = [
        (
            format!("uuid: {original}"),
            format!(
                "uuid: \"\\x{:02x}{}\"",
                original.as_bytes()[0],
                &original[1..]
            ),
        ),
        (format!("uuid: {BUILD}"), format!("uuid: '{BUILD}'")),
        (
            format!("uuid: {SOUND}"),
            format!("uuid: |-\n        {SOUND}"),
        ),
    ];
    fs::write(&file, rewritten(NES_DEV, &changes)).unwrap();

    let copied = import(&file, &library).unwrap();

    assert_eq!(copied.copy_of.as_deref(), Some(original.as_str()));
    let copy = library.open(&copied.uuid).unwrap();
    assert!(uuids(&copy).is_disjoint(&before), "{:?}", uuids(&copy));
    assert_eq!(uuids(&copy).len(), before.len());
    assert_eq!(copy.program.title, "Разработка игр для NES");
    assert_eq!(copy.program.map.children[1].title, "Звук и музыка");
}
