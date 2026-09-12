#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "library gate: a panic here is the report"
)]

mod support;

use std::fs;

use support::programs::{plant, scratch, snapshot};
use tolearn_core::library::{Library, LibraryError};
use tolearn_core::program;

const NES_DEV: &str = "fixtures/v2/valid/nes-dev";

#[test]
fn a_copied_program_loads_as_the_one_in_the_library_with_its_subprograms() {
    let data = scratch("copy");
    let uuid = plant(&data, NES_DEV);
    let library = Library::at(&data);
    let tree = library.open(&uuid).unwrap();
    let before = snapshot(&data.join("programs"));
    let to = data.join("copied");

    library.copy(&tree, &to).unwrap();

    assert!(!tree.children.is_empty());
    assert_eq!(program::load(&to).unwrap(), tree);
    assert_eq!(snapshot(&data.join("programs")), before);
}

#[test]
fn a_program_gone_from_the_library_is_not_copied() {
    let data = scratch("copy-gone");
    let uuid = plant(&data, NES_DEV);
    let library = Library::at(&data);
    let tree = library.open(&uuid).unwrap();
    fs::remove_dir_all(data.join("programs").join(&uuid)).unwrap();

    let error = library.copy(&tree, &data.join("copied")).unwrap_err();

    assert!(
        matches!(error, LibraryError::Unreadable { .. }),
        "{error:?}"
    );
}

#[test]
fn a_tree_pointing_outside_the_library_is_not_copied() {
    let data = scratch("copy-outside");
    let uuid = plant(&data, NES_DEV);
    let library = Library::at(&data);
    let mut tree = library.open(&uuid).unwrap();
    tree.program.uuid = "..".to_owned();
    let to = data.join("copied");

    let error = library.copy(&tree, &to).unwrap_err();

    assert!(matches!(error, LibraryError::Absent { .. }), "{error:?}");
    assert!(!to.exists());
}
