#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "package import gate: a panic here is the report"
)]

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::packages::{CHIPTUNE, NES_DEV};
use support::programs::{listing, scratch, snapshot, uuid_of};
use support::sealing::packed;
use tolearn_core::library::{self, Library};
use tolearn_core::package::{Imported, import, unpack};

fn contents(directory: &Path) -> Vec<(String, Vec<u8>)> {
    library::read(directory)
        .unwrap()
        .files()
        .into_iter()
        .map(|file| {
            let data = fs::read(directory.join(&file)).unwrap();
            (file, data)
        })
        .collect()
}

#[test]
fn a_packed_program_comes_back_the_same_under_the_same_uuid() {
    for (name, relative) in [("chiptune", CHIPTUNE), ("nes-dev", NES_DEV)] {
        let room = scratch(&format!("import-round-{name}"));
        let data = room.join("data");
        let library = Library::at(&data);
        let source = support::root().join(relative);
        let uuid = uuid_of(relative);

        let imported = import(&packed(&room, relative), &library).unwrap();

        assert_eq!(
            imported,
            Imported {
                uuid: uuid.clone(),
                copy_of: None
            }
        );
        let installed = data.join("programs").join(&uuid);
        assert_eq!(
            library.open(&uuid).unwrap(),
            library::read(&source).unwrap()
        );
        assert_eq!(contents(&installed), contents(&source));
        assert_eq!(snapshot(&installed).len(), contents(&source).len());
        assert_eq!(listing(&data.join("programs")), [uuid]);
    }
}

#[test]
fn a_package_unpacks_into_a_new_or_empty_directory_as_the_program_it_holds() {
    let room = scratch("unpack-directory");
    let file = packed(&room, NES_DEV);
    let into = room.join("nes");

    let tree = unpack(&file, &into).unwrap();

    assert_eq!(tree.program.uuid, uuid_of(NES_DEV));
    assert_eq!(contents(&into), contents(&support::root().join(NES_DEV)));
    assert_eq!(snapshot(&into).len(), contents(&into).len());
    assert_eq!(listing(&room), ["nes", "program.tolearn"]);

    let empty = room.join("empty");
    fs::create_dir(&empty).unwrap();
    unpack(&file, &empty).unwrap();
    assert_eq!(contents(&empty), contents(&into));
}

#[test]
fn unpack_leaves_a_directory_that_holds_anything_alone() {
    let room = scratch("unpack-occupied");
    let file = packed(&room, CHIPTUNE);
    let into = room.join("busy");
    fs::create_dir_all(&into).unwrap();
    fs::write(into.join("keep.txt"), "моё").unwrap();

    let error = unpack(&file, &into).unwrap_err();

    assert_eq!(error.code(), "package.occupied");
    assert!(error.to_string().contains("busy"), "{error}");
    assert_eq!(
        snapshot(&into),
        [(PathBuf::from("keep.txt"), "моё".as_bytes().to_vec())]
    );
    assert_eq!(listing(&room), ["busy", "program.tolearn"]);
}

#[test]
fn unpack_wants_the_directory_above_its_target_to_exist() {
    let room = scratch("unpack-no-parent");
    let file = packed(&room, CHIPTUNE);

    let error = unpack(&file, &room.join("absent/chiptune")).unwrap_err();

    assert_eq!(error.code(), "package.unwritable");
    assert!(error.to_string().contains("absent"), "{error}");
    assert_eq!(listing(&room), ["program.tolearn"]);
}
