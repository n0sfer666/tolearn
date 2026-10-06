#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "asset gate: a panic here is the report"
)]

mod support;

use std::fs;

use support::programs::scratch;
use tolearn_core::library::{Library, LibraryError};

const CHIPTUNE: &str = "examples/chiptune";
const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
const ROM: &str =
    "children/b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56/children/e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

fn original(relative: &str, file: &str) -> Vec<u8> {
    fs::read(support::root().join(relative).join(file)).unwrap()
}

#[test]
fn an_asset_of_an_installed_program_reads_byte_for_byte() {
    let library = Library::at(&scratch("asset-root"));
    let uuid = library.install(&support::root().join(CHIPTUNE)).unwrap();
    let tree = library.open(&uuid).unwrap();

    for file in ["assets/pulse-wave.png", "assets/voices.svg"] {
        assert_eq!(
            library.asset(&tree, file).unwrap(),
            original(CHIPTUNE, file),
            "{file}"
        );
    }
}

#[test]
fn a_nested_asset_reads_through_its_branch_prefix() {
    let library = Library::at(&scratch("asset-nested"));
    let uuid = library.install(&support::root().join(NES_DEV)).unwrap();
    let tree = library.open(&uuid).unwrap();
    let file = format!("{ROM}/assets/pixel.png");

    assert_eq!(
        library.asset(&tree, &file).unwrap(),
        original(NES_DEV, &file)
    );
}

#[test]
fn a_path_outside_the_tree_files_is_refused() {
    let library = Library::at(&scratch("asset-foreign"));
    let uuid = library.install(&support::root().join(CHIPTUNE)).unwrap();
    let tree = library.open(&uuid).unwrap();

    for file in [
        "assets/../program.yaml",
        "stages/envelope.yaml",
        "assets/missing.png",
        "/etc/passwd",
        "../other/program.yaml",
        "",
    ] {
        let error = library.asset(&tree, file).unwrap_err();
        assert_eq!(error.code(), "library.foreign", "{file}");
        assert!(
            matches!(&error, LibraryError::Foreign { uuid: owner, file: named } if owner == &uuid && named == file),
            "{error:?}"
        );
    }
}

#[test]
fn a_listed_asset_gone_from_disk_is_unreadable() {
    let data = scratch("asset-gone");
    let library = Library::at(&data);
    let uuid = library.install(&support::root().join(CHIPTUNE)).unwrap();
    let tree = library.open(&uuid).unwrap();
    fs::remove_file(data.join("programs").join(&uuid).join("assets/voices.svg")).unwrap();

    let error = library.asset(&tree, "assets/voices.svg").unwrap_err();

    assert_eq!(error.code(), "library.unreadable");
}

#[cfg(unix)]
#[test]
fn a_listed_asset_linked_outside_the_program_is_foreign() {
    let data = scratch("asset-link");
    let library = Library::at(&data);
    let uuid = library.install(&support::root().join(CHIPTUNE)).unwrap();
    let tree = library.open(&uuid).unwrap();
    let outside = data.join("secret.svg");
    fs::write(&outside, "<svg/>").unwrap();
    let listed = data.join("programs").join(&uuid).join("assets/voices.svg");
    fs::remove_file(&listed).unwrap();
    std::os::unix::fs::symlink(&outside, &listed).unwrap();

    let error = library.asset(&tree, "assets/voices.svg").unwrap_err();

    assert_eq!(error.code(), "library.foreign", "{error}");
}
