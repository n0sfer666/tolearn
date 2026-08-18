#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "speech gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

const FORBIDDEN: [&str; 6] = [
    "File::create",
    "fs::write",
    "OpenOptions",
    "TcpStream",
    "std::net",
    "reqwest",
];

fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().and_then(|kind| kind.to_str()) == Some("rs") {
            found.push(path);
        }
    }
}

#[test]
fn запись_никуда_не_утекает_из_памяти() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    assert!(!files.is_empty(), "{}: нет исходников", src.display());

    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        for marker in FORBIDDEN {
            assert!(
                !text.contains(marker),
                "{}: {marker} — записанный голос не должен покидать память",
                path.display()
            );
        }
    }
}
