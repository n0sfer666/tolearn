use std::collections::HashSet;
use std::fs::{self, Metadata};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::error::SearchError;
use super::types::{Seen, Stamp};
use crate::library::Library;
use crate::yaml::is_uuid;

#[derive(Debug, Clone)]
pub struct Wanted {
    pub program: String,
    pub files: Vec<Seen>,
}

pub fn plan(library: &Library) -> Result<Vec<Wanted>, SearchError> {
    let root = library.root();
    let listing = match fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(unreadable(root, &error)),
    };
    let mut wanted = Vec::new();
    for item in listing {
        let path = item.map_err(|error| unreadable(root, &error))?.path();
        let Some(program) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !is_uuid(program) || !path.is_dir() {
            continue;
        }
        if let Ok(files) = files(&path) {
            wanted.push(Wanted {
                program: program.to_owned(),
                files,
            });
        }
    }
    wanted.sort_by(|left, right| left.program.cmp(&right.program));
    Ok(wanted)
}

fn files(home: &Path) -> io::Result<Vec<Seen>> {
    let mut files = Vec::new();
    walk(home, "", &mut HashSet::new(), &mut files)?;
    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(files)
}

fn walk(
    directory: &Path,
    prefix: &str,
    visited: &mut HashSet<PathBuf>,
    files: &mut Vec<Seen>,
) -> io::Result<()> {
    if !visited.insert(fs::canonicalize(directory)?) {
        return Ok(());
    }
    for item in fs::read_dir(directory)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let path = item.path();
        let data = match fs::metadata(&path) {
            Ok(data) => data,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let relative = format!("{prefix}{name}");
        if data.is_dir() {
            walk(&path, &format!("{relative}/"), visited, files)?;
        } else if data.is_file() {
            files.push(Seen {
                name: relative,
                stamp: stamp(&data),
            });
        }
    }
    Ok(())
}

fn stamp(data: &Metadata) -> Stamp {
    let modified = data
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .unwrap_or_default();
    Stamp {
        modified_nanos: modified.as_nanos(),
        size: data.len(),
    }
}

fn unreadable(path: &Path, error: &io::Error) -> SearchError {
    SearchError::Unreadable {
        path: path.display().to_string(),
        reason: error.to_string(),
    }
}
