#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "library gate: a panic here is the report"
)]

mod support;

use std::fs;

use support::programs::{copy_tree, plant, scratch};
use tolearn_core::library::{Entry, Library, LibraryError, Refusal};
use tolearn_core::program;

const CHIPTUNE: &str = "examples/chiptune";
const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
const TOO_DEEP: &str = "fixtures/v2/invalid/program.too-deep__four-levels";
const MALFORMED: &str = "fixtures/v2/invalid/yaml.wrong-type__stage-title-a-list";
const STRANGER: &str = "0badc0de-0000-4000-8000-000000000000";

fn verdicts(entries: &[Entry]) -> Vec<(String, &'static str)> {
    entries
        .iter()
        .map(|entry| {
            let code = match &entry.program {
                Ok(_) => "ok",
                Err(refusal) => refusal.code(),
            };
            (entry.directory.clone(), code)
        })
        .collect()
}

fn sorted(mut expected: Vec<(String, &'static str)>) -> Vec<(String, &'static str)> {
    expected.sort();
    expected
}

#[test]
fn a_library_without_its_directory_lists_nothing() {
    let data = scratch("library-empty");
    assert_eq!(Library::at(&data).list().unwrap(), Vec::<Entry>::new());
}

#[test]
fn every_program_directory_is_listed_and_a_broken_one_carries_its_reason() {
    let data = scratch("library-list");
    let good = plant(&data, CHIPTUNE);
    let deep = plant(&data, TOO_DEEP);
    let malformed = plant(&data, MALFORMED);
    let programs = data.join("programs");
    copy_tree(&support::root().join(CHIPTUNE), &programs.join(STRANGER));
    fs::write(programs.join("notes.txt"), "не программа").unwrap();

    let entries = Library::at(&data).list().unwrap();

    assert_eq!(
        verdicts(&entries),
        sorted(vec![
            (good, "ok"),
            (deep.clone(), "library.invalid"),
            (malformed, "yaml.wrong-type"),
            (STRANGER.to_owned(), "library.misplaced"),
        ])
    );
    let too_deep = entries
        .iter()
        .find(|entry| entry.directory == deep)
        .unwrap();
    let Err(Refusal::Invalid(violations)) = &too_deep.program else {
        panic!("{too_deep:?}");
    };
    let codes: Vec<&str> = violations
        .iter()
        .map(|violation| violation.code())
        .collect();
    assert_eq!(codes, ["program.too-deep"]);
}

#[cfg(unix)]
#[test]
fn a_program_the_library_cannot_read_is_marked_and_does_not_break_the_list() {
    let data = scratch("library-unreadable");
    let good = plant(&data, CHIPTUNE);
    let locked = plant(&data, NES_DEV);
    let file = data.join("programs").join(&locked).join("program.yaml");

    support::programs::chmod(&file, 0o000);
    let entries = Library::at(&data).list();
    support::programs::chmod(&file, 0o644);

    assert_eq!(
        verdicts(&entries.unwrap()),
        sorted(vec![(good, "ok"), (locked, "program.unreadable")])
    );
}

#[test]
fn a_program_opens_by_its_uuid_as_it_reads_in_place() {
    let data = scratch("library-open");
    let uuid = plant(&data, CHIPTUNE);
    let opened = Library::at(&data).open(&uuid).unwrap();
    assert_eq!(
        opened,
        program::load(&support::root().join(CHIPTUNE)).unwrap()
    );
}

#[test]
fn a_broken_program_opens_into_its_reason() {
    let data = scratch("library-open-broken");
    let uuid = plant(&data, MALFORMED);
    let error = Library::at(&data).open(&uuid).unwrap_err();
    assert_eq!(error.code(), "yaml.wrong-type");
}

#[test]
fn a_uuid_the_library_does_not_hold_is_absent_and_never_a_path() {
    let data = scratch("library-absent");
    plant(&data, CHIPTUNE);
    let library = Library::at(&data);
    for uuid in [
        STRANGER,
        "",
        ".",
        "..",
        "../programs",
        "/etc",
        "0BADC0DE-0000-4000-8000-000000000000",
    ] {
        let absent = LibraryError::Absent {
            uuid: uuid.to_owned(),
        };
        assert_eq!(library.open(uuid).unwrap_err(), absent, "{uuid}");
    }
}
