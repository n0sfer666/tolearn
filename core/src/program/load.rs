use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use super::MAX_DEPTH;
use super::error::LoadError;
use super::parse::parse;
use super::tree::Tree;
use crate::stage;
use crate::yaml::ParseError;

pub fn load(directory: &Path) -> Result<Tree, LoadError> {
    node(directory, 1)
}

fn node(directory: &Path, depth: usize) -> Result<Tree, LoadError> {
    let program = file(&directory.join("program.yaml"), parse)?;
    let mut stages = BTreeMap::new();
    for row in &program.map.stages {
        let path = directory.join("stages").join(format!("{}.yaml", row.id));
        if path.is_file() {
            stages.insert(row.id.clone(), file(&path, stage::parse)?);
        }
    }
    let mut children = BTreeMap::new();
    if depth < MAX_DEPTH {
        for row in &program.map.children {
            let path = directory.join("children").join(&row.uuid);
            if path.is_dir() {
                children.insert(row.uuid.clone(), node(&path, depth + 1)?);
            }
        }
    }
    Ok(Tree {
        assets: assets(&directory.join("assets"))?,
        program,
        stages,
        children,
    })
}

fn file<T>(path: &Path, parse: impl Fn(&str) -> Result<T, ParseError>) -> Result<T, LoadError> {
    let source = fs::read_to_string(path).map_err(|error| unreadable(path, &error))?;
    parse(&source).map_err(|error| LoadError::Malformed {
        path: path.to_owned(),
        error,
    })
}

fn assets(folder: &Path) -> Result<BTreeSet<String>, LoadError> {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(error) => return Err(unreadable(folder, &error)),
    };
    let mut found = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| unreadable(folder, &error))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_file() && !name.starts_with('.') {
            found.insert(format!("assets/{name}"));
        }
    }
    Ok(found)
}

fn unreadable(path: &Path, error: &io::Error) -> LoadError {
    LoadError::Unreadable {
        path: path.to_owned(),
        kind: error.kind(),
    }
}
