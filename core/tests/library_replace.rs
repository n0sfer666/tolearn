#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "library gate: a panic here is the report"
)]

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::programs::{copy_tree, listing, plant, scratch, snapshot, uuid_of};
use tolearn_core::library::{Library, LibraryError};
use tolearn_core::program;

const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
const RETITLED: &str = "NES с нуля, второе издание";
const CHIPTUNE: &str = "examples/chiptune";

fn retitled(data: &Path) -> PathBuf {
    let source = data.join("source");
    copy_tree(&support::root().join(NES_DEV), &source);
    let file = source.join("program.yaml");
    let mut program = program::parse(&fs::read_to_string(&file).unwrap()).unwrap();
    program.title = RETITLED.to_owned();
    fs::write(&file, program::write(&program).unwrap()).unwrap();
    source
}

fn nes_dev() -> program::Tree {
    program::load(&support::root().join(NES_DEV)).unwrap()
}

#[test]
fn a_replaced_program_opens_as_its_new_source_and_leaves_nothing_behind() {
    let data = scratch("replace");
    let uuid = plant(&data, NES_DEV);
    let source = retitled(&data);
    let library = Library::at(&data);

    assert_eq!(library.replace(&source).unwrap(), uuid);

    assert_eq!(
        library.open(&uuid).unwrap(),
        program::load(&source).unwrap()
    );
    assert_eq!(listing(&data.join("programs")), [uuid]);
}

#[test]
fn an_unreadable_source_is_refused_and_the_program_stays_as_it_was() {
    let data = scratch("replace-refused");
    plant(&data, NES_DEV);
    let source = retitled(&data);
    fs::write(source.join("program.yaml"), "schema: tolearn/program/1\n").unwrap();
    let before = snapshot(&data.join("programs"));

    let error = Library::at(&data).replace(&source).unwrap_err();

    assert!(matches!(error, LibraryError::Refused(_)), "{error:?}");
    assert_eq!(snapshot(&data.join("programs")), before);
}

#[test]
fn only_a_program_in_the_library_is_replaced() {
    let data = scratch("replace-absent");
    let source = retitled(&data);

    let error = Library::at(&data).replace(&source).unwrap_err();

    assert_eq!(
        error,
        LibraryError::Absent {
            uuid: uuid_of(NES_DEV)
        }
    );
    assert!(!data.join("programs").exists());
}

#[test]
fn a_swap_cut_between_its_renames_is_undone_by_the_next_write_and_not_by_reading() {
    let data = scratch("replace-between");
    let uuid = uuid_of(NES_DEV);
    let programs = data.join("programs");
    let cut = [format!(".{uuid}.7-0.old"), format!(".{uuid}.7-0.tmp")];
    copy_tree(&support::root().join(NES_DEV), &programs.join(&cut[0]));
    copy_tree(&retitled(&data), &programs.join(&cut[1]));
    let library = Library::at(&data);

    assert!(library.list().unwrap().is_empty());
    assert!(
        matches!(library.open(&uuid), Err(LibraryError::Absent { .. })),
        "чтение подняло недописанную программу"
    );
    assert_eq!(listing(&programs), cut);

    library.install(&support::root().join(CHIPTUNE)).unwrap();

    assert_eq!(library.open(&uuid).unwrap(), nes_dev());
    assert_eq!(library.list().unwrap().len(), 2);
}

#[test]
fn a_swap_cut_after_its_renames_keeps_the_new_program() {
    let data = scratch("replace-after");
    let uuid = uuid_of(NES_DEV);
    let programs = data.join("programs");
    let old = format!(".{uuid}.7-0.old");
    copy_tree(&retitled(&data), &programs.join(&uuid));
    copy_tree(&support::root().join(NES_DEV), &programs.join(&old));
    let library = Library::at(&data);

    let entries = library.list().unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].program.as_ref().unwrap().program.title, RETITLED);
    assert_eq!(listing(&programs), [old, uuid.clone()]);
    let other = library.install(&support::root().join(CHIPTUNE)).unwrap();
    let mut left = vec![uuid, other];
    left.sort();
    assert_eq!(listing(&programs), left);
}

#[test]
fn replaces_racing_in_one_process_all_land() {
    let data = scratch("replace-race");
    let uuid = plant(&data, NES_DEV);
    let source = retitled(&data);
    let library = Library::at(&data);

    let results: Vec<_> = std::thread::scope(|scope| {
        let runs: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| (0..5).map(|_| library.replace(&source)).collect::<Vec<_>>()))
            .collect();
        runs.into_iter()
            .flat_map(|run| run.join().unwrap())
            .collect()
    });

    assert!(
        results.iter().all(|result| result.as_ref() == Ok(&uuid)),
        "{results:?}"
    );
    assert_eq!(listing(&data.join("programs")), [uuid]);
}

#[test]
fn a_swap_cut_before_its_renames_keeps_the_old_program() {
    let data = scratch("replace-before");
    let uuid = plant(&data, NES_DEV);
    let programs = data.join("programs");
    copy_tree(&retitled(&data), &programs.join(format!(".{uuid}.7-0.tmp")));

    assert_eq!(Library::at(&data).open(&uuid).unwrap(), nes_dev());
    assert_eq!(Library::at(&data).list().unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn a_library_that_takes_no_writes_keeps_the_program_whole() {
    let data = scratch("replace-locked");
    plant(&data, NES_DEV);
    let source = retitled(&data);
    let programs = data.join("programs");
    let before = snapshot(&programs);
    support::programs::chmod(&programs, 0o555);

    let error = Library::at(&data).replace(&source).unwrap_err();

    support::programs::chmod(&programs, 0o755);
    assert!(
        matches!(error, LibraryError::Unwritable { .. }),
        "{error:?}"
    );
    assert_eq!(snapshot(&programs), before);
}
