#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "state gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeMap;
use std::path::Path;

use support::read;
use support::states::{PROGRAM, full, scratch};
use tolearn_core::state::{self, State};

#[test]
fn правка_пишет_только_в_state_uuid() {
    let data = scratch("layout");
    let program = data.join("programs").join(PROGRAM);
    std::fs::create_dir_all(program.join("stages")).unwrap();
    std::fs::write(program.join("program.yaml"), "schema: tolearn/program/1\n").unwrap();
    std::fs::write(
        program.join("stages/voices.yaml"),
        "schema: tolearn/stage/1\n",
    )
    .unwrap();
    let before = snapshot(&data);

    State::update(&data, PROGRAM, |state| *state = full()).unwrap();

    let after = snapshot(&data);
    let changed: Vec<&String> = after
        .iter()
        .filter(|(path, bytes)| before.get(*path) != Some(bytes))
        .map(|(path, _)| path)
        .collect();
    assert_eq!(changed, vec![&format!("state/{PROGRAM}/state.yaml")]);
    assert!(
        before.keys().all(|path| after.contains_key(path)),
        "a file disappeared from the data directory"
    );
}

#[test]
fn корпус_схемы_состояния_записан_самим_приложением() {
    for (fixture, written) in [
        ("fixtures/v2/states/full.yaml", full()),
        ("fixtures/v2/states/empty.yaml", State::new(PROGRAM)),
    ] {
        assert_eq!(
            state::render(&written).unwrap(),
            read(fixture),
            "{fixture}: the app writes this state differently, \
             so the schema gate checks a file the app never writes"
        );
        assert_eq!(state::parse(&read(fixture)).unwrap(), written);
    }
}

fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    walk(root, root, &mut found);
    found
}

fn walk(root: &Path, directory: &Path, found: &mut BTreeMap<String, Vec<u8>>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(root, &path, found);
            continue;
        }
        let relative = path.strip_prefix(root).unwrap().to_string_lossy();
        found.insert(relative.replace('\\', "/"), std::fs::read(&path).unwrap());
    }
}
