use super::corpus::listed;
use crate::repo::root;

pub const REFERENCE: &str = "examples/chiptune";

const VALID: &str = "fixtures/v2/valid";

pub fn roots() -> Vec<String> {
    std::iter::once(REFERENCE.to_owned())
        .chain(directories(VALID))
        .collect()
}

pub fn children(program: &str) -> Vec<String> {
    directories(&format!("{program}/children"))
}

pub fn documents(kind: &str) -> Vec<String> {
    let mut found = Vec::new();
    for program in roots() {
        collect(&program, kind, &mut found);
    }
    found
}

fn collect(program: &str, kind: &str, found: &mut Vec<String>) {
    match kind {
        "program" => found.push(format!("{program}/program.yaml")),
        "stage" => found.extend(listed(&format!("{program}/stages"))),
        other => panic!("unknown v2 schema kind `{other}`"),
    }
    for child in children(program) {
        collect(&child, kind, found);
    }
}

fn directories(relative: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root().join(relative)) else {
        return Vec::new();
    };
    let mut found: Vec<String> = entries
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| format!("{relative}/{}", entry.file_name().to_string_lossy()))
        .collect();
    found.sort();
    found
}
