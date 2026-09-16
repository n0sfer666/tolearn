#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "repository gate: a panic here is the report"
)]

mod repo;

#[path = "../../tests-support/scratch.rs"]
mod scratch;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

const CRATES: &[&str] = &[
    "core", "runner", "offline", "provider", "generate", "gestures", "speech", "cli", "app",
];
const MARK: &str = concat!("env::", "temp_dir()");
const SPARED: &[&str] = &[
    "provider/src/scratch.rs",
    "app/examples/mermaid.rs",
    "tests-support/scratch.rs",
];

#[test]
fn a_swept_root_keeps_only_the_rooms_of_living_processes() {
    let room = scratch::made("gate-sweep");
    let finished = Command::new("true").spawn().unwrap();
    let gone = finished.id();
    finished.wait_with_output().unwrap();

    let mine = room.join(format!("{}-mine", std::process::id()));
    let theirs = room.join(format!("{gone}-theirs"));
    let unnamed = room.join("noise");
    for path in [&mine, &theirs, &unnamed] {
        fs::create_dir_all(path).unwrap();
    }

    scratch::sweep(&room);

    assert!(mine.is_dir(), "{}", mine.display());
    assert!(!theirs.exists(), "{}", theirs.display());
    assert!(unnamed.is_dir(), "{}", unnamed.display());
}

#[test]
fn a_scratch_room_lives_under_one_root_named_after_its_process() {
    let room = scratch::made("gate-room");

    assert_eq!(room.parent(), Some(scratch::root().as_path()));
    assert_eq!(
        room.file_name().and_then(|name| name.to_str()),
        Some(format!("{}-gate-room", std::process::id()).as_str())
    );
    assert!(room.is_dir(), "{}", room.display());
}

#[test]
fn a_named_room_is_not_created_until_asked() {
    let room = scratch::named("gate-named");

    assert!(!room.exists(), "{}", room.display());
    assert_eq!(room.parent(), Some(scratch::root().as_path()));
}

#[test]
fn a_reused_label_starts_from_an_empty_room() {
    let room = scratch::made("gate-reuse");
    fs::write(room.join("stale"), b"old").unwrap();

    let again = scratch::made("gate-reuse");

    assert_eq!(again, room);
    assert!(!again.join("stale").exists());
}

#[test]
fn no_test_builds_its_own_temporary_directory() {
    let spared: BTreeSet<&str> = SPARED.iter().copied().collect();
    let guilty: Vec<String> = sources()
        .into_iter()
        .filter(|(relative, _)| !spared.contains(relative.as_str()))
        .filter(|(_, text)| text.contains(MARK))
        .map(|(relative, _)| relative)
        .collect();

    assert!(guilty.is_empty(), "{guilty:?}");
}

fn sources() -> Vec<(String, String)> {
    let root = repo::root();
    let mut found = Vec::new();
    for directory in CRATES {
        collect(&root.join(directory), &root, &mut found);
    }
    found
}

fn collect(from: &Path, root: &Path, found: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(from) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, root, found);
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            let relative = relative(&path, root);
            let text = repo::read(&relative);
            found.push((relative, text));
        }
    }
}

fn relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
