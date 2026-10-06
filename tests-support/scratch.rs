#![allow(
    dead_code,
    clippy::unwrap_used,
    reason = "test scratch: shared by test targets, a panic here is the report"
)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;

const ROOT: &str = "tolearn-tests";

pub fn made(label: &str) -> PathBuf {
    let path = named(label);
    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn named(label: &str) -> PathBuf {
    let path = root().join(format!("{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

pub fn root() -> PathBuf {
    static SWEPT: Once = Once::new();
    let root = std::env::temp_dir().join(ROOT);
    SWEPT.call_once(|| sweep(&root));
    root
}

pub fn sweep(root: &Path) {
    let Some(living) = living() else { return };
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(owner) = name.to_str().and_then(owner) else {
            continue;
        };
        if !living.contains(&owner) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

fn owner(name: &str) -> Option<u32> {
    name.split('-').next()?.parse().ok()
}

fn living() -> Option<BTreeSet<u32>> {
    let output = Command::new("ps").args(["-Ao", "pid="]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .filter_map(|word| word.parse().ok())
            .collect(),
    )
}
