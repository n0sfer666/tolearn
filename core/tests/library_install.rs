#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "library gate: a panic here is the report"
)]

mod support;

use std::fs;

use support::programs::{copy_tree, scratch, snapshot, uuid_of};
use tolearn_core::library::{Library, LibraryError};
use tolearn_core::program;

const CHIPTUNE: &str = "examples/chiptune";
const NES_DEV: &str = "fixtures/v2/valid/nes-dev";

#[test]
fn an_installed_program_is_listed_and_opens_as_its_source_reads() {
    let data = scratch("install");
    let library = Library::at(&data);
    let source = support::root().join(NES_DEV);

    let uuid = library.install(&source).unwrap();

    assert_eq!(uuid, uuid_of(NES_DEV));
    let listed: Vec<String> = library
        .list()
        .unwrap()
        .into_iter()
        .map(|entry| entry.directory)
        .collect();
    assert_eq!(listed, [uuid.as_str()]);
    assert_eq!(
        library.open(&uuid).unwrap(),
        program::load(&source).unwrap()
    );
}

#[test]
fn only_what_the_reading_takes_is_installed_byte_for_byte() {
    let data = scratch("install-content");
    let source = data.join("source");
    copy_tree(&support::root().join(CHIPTUNE), &source);
    let expected = snapshot(&source);
    fs::write(source.join("notes.txt"), "черновик").unwrap();
    fs::write(
        source.join("stages/orphan.yaml"),
        "schema: tolearn/stage/1\n",
    )
    .unwrap();

    let uuid = Library::at(&data).install(&source).unwrap();

    assert_eq!(snapshot(&data.join("programs").join(uuid)), expected);
}

#[test]
fn a_taken_uuid_is_refused_and_the_installed_program_stays_whole() {
    let data = scratch("install-taken");
    let library = Library::at(&data);
    let uuid = library.install(&support::root().join(CHIPTUNE)).unwrap();
    let before = snapshot(&data);

    let error = library
        .install(&support::root().join(CHIPTUNE))
        .unwrap_err();

    assert_eq!(error, LibraryError::Taken { uuid });
    assert_eq!(snapshot(&data), before);
}

#[test]
fn a_program_that_does_not_read_is_not_installed() {
    let data = scratch("install-refused");
    let library = Library::at(&data);
    for (fixture, code) in [
        (
            "fixtures/v2/invalid/program.too-deep__four-levels",
            "library.invalid",
        ),
        (
            "fixtures/v2/invalid/yaml.wrong-type__stage-title-a-list",
            "yaml.wrong-type",
        ),
    ] {
        let error = library.install(&support::root().join(fixture)).unwrap_err();
        assert_eq!(error.code(), code, "{fixture}");
    }
    assert!(!data.join("programs").exists());
}

#[cfg(unix)]
#[test]
fn an_install_broken_off_midway_leaves_the_library_as_it_was() {
    let data = scratch("install-broken-off");
    let library = Library::at(&data);
    library.install(&support::root().join(CHIPTUNE)).unwrap();
    let source = data.join("source");
    copy_tree(&support::root().join(NES_DEV), &source);
    let picture = source.join(
        "children/b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56/children/e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47/assets/pixel.png",
    );
    let before = snapshot(&data.join("programs"));

    support::programs::chmod(&picture, 0o000);
    let error = library.install(&source);
    support::programs::chmod(&picture, 0o644);

    assert_eq!(
        error.unwrap_err(),
        LibraryError::Unreadable {
            path: picture,
            kind: std::io::ErrorKind::PermissionDenied,
        }
    );
    assert_eq!(snapshot(&data.join("programs")), before);
    assert_eq!(fs::read_dir(data.join("programs")).unwrap().count(), 1);
}

#[test]
fn what_a_crashed_install_left_behind_is_no_program() {
    let data = scratch("install-crashed");
    let uuid = uuid_of(NES_DEV);
    let leftover = data.join("programs").join(format!(".{uuid}.1.tmp"));
    copy_tree(&support::root().join(NES_DEV), &leftover);
    let library = Library::at(&data);

    assert_eq!(library.list().unwrap().len(), 0);
    assert_eq!(
        library.open(&uuid).unwrap_err(),
        LibraryError::Absent { uuid: uuid.clone() }
    );
    assert_eq!(
        library.install(&support::root().join(NES_DEV)).unwrap(),
        uuid
    );
}

#[cfg(unix)]
#[test]
fn files_of_v1_beside_the_library_are_neither_read_nor_changed() {
    let data = scratch("install-v1");
    copy_tree(
        &support::root().join("examples/llm-agents-base"),
        &data.join("unpacked/llm-agents-base"),
    );
    for (path, text) in [
        ("history/llm-agents-base/2026-09-01.yaml", "versions: []\n"),
        ("offline/objects/ab/cdef", "страница"),
        ("draft/roadmap.yaml", "черновик"),
        ("search-ru.yaml", "index: []\n"),
    ] {
        let path = data.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let before = snapshot(&data);
    let locked = ["unpacked", "history", "offline", "draft"];

    for name in locked {
        support::programs::chmod(&data.join(name), 0o000);
    }
    let library = Library::at(&data);
    let installed = library.install(&support::root().join(CHIPTUNE));
    let listed = library.list();
    for name in locked {
        support::programs::chmod(&data.join(name), 0o755);
    }

    let uuid = installed.unwrap();
    assert_eq!(library.open(&uuid).unwrap().program.uuid, uuid);
    assert_eq!(listed.unwrap().len(), 1);
    let after: Vec<_> = snapshot(&data)
        .into_iter()
        .filter(|(path, _)| !path.starts_with("programs"))
        .collect();
    assert_eq!(after, before);
}
