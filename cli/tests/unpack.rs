#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

#[allow(
    dead_code,
    reason = "the unpack gate takes only its part of the shared support module"
)]
mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::{code, json, scratch, stderr, tolearn};
use tolearn_core::package::pack;

fn repo(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(relative)
}

fn packed(room: &Path) -> PathBuf {
    let file = room.join("chiptune.tolearn");
    fs::write(&file, pack(&repo("examples/chiptune")).unwrap()).unwrap();
    file
}

#[test]
fn unpack_opens_what_pack_wrote() {
    let room = scratch("unpack");
    let file = packed(&room);
    let into = room.join("chiptune");

    let out = tolearn(&[
        "unpack",
        file.to_str().unwrap(),
        into.to_str().unwrap(),
        "--json",
    ]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let report = json(&out);
    assert_eq!(report["path"], into.display().to_string());
    assert_eq!(report["uuid"], "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84");
    for name in ["program.yaml", "stages/voices.yaml", "assets/voices.svg"] {
        assert_eq!(
            fs::read(into.join(name)).unwrap(),
            fs::read(repo("examples/chiptune").join(name)).unwrap(),
            "{name}"
        );
    }
    assert!(!into.join("manifest.json").exists());
}

#[test]
fn a_broken_package_is_refused_with_its_code_and_nothing_appears() {
    let room = scratch("unpack-broken");
    let file = room.join("broken.tolearn");
    fs::write(&file, "program.yaml\n").unwrap();
    let into = room.join("broken");

    let out = tolearn(&["unpack", file.to_str().unwrap(), into.to_str().unwrap()]);

    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("archive.unknown-format"),
        "{}",
        stderr(&out)
    );
    assert!(!into.exists());
    assert_eq!(fs::read_dir(&room).unwrap().count(), 1);
}

#[test]
fn a_directory_that_holds_files_is_left_alone() {
    let room = scratch("unpack-occupied");
    let file = packed(&room);
    let into = room.join("busy");
    fs::create_dir_all(&into).unwrap();
    fs::write(into.join("keep.txt"), "моё").unwrap();

    let out = tolearn(&["unpack", file.to_str().unwrap(), into.to_str().unwrap()]);

    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("package.occupied"),
        "{}",
        stderr(&out)
    );
    assert_eq!(fs::read_dir(&into).unwrap().count(), 1);
}

#[test]
fn unpack_without_the_directory_is_a_usage_error() {
    let room = scratch("unpack-usage");
    let file = packed(&room);

    let out = tolearn(&["unpack", file.to_str().unwrap()]);

    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("`unpack`"), "{}", stderr(&out));
}
