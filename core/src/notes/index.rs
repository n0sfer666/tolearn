use std::fs::{DirEntry, read_dir};
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::error::NoteError;
use super::frontmatter::parse;
use super::types::{Note, Stamp};

pub fn index(root: &Path) -> Result<Vec<Note>, NoteError> {
    let mut found = Vec::new();
    walk(root, &mut found)?;
    found.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(found)
}

fn walk(directory: &Path, found: &mut Vec<Note>) -> Result<(), NoteError> {
    let listing = match read_dir(directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(unreadable(directory, &error)),
    };

    for entry in listing {
        let entry = entry.map_err(|error| unreadable(directory, &error))?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found)?;
        } else if markdown(&path)
            && let Some(note) = note(&entry)?
        {
            found.push(note);
        }
    }
    Ok(())
}

fn markdown(path: &Path) -> bool {
    path.extension().is_some_and(|kind| kind == "md")
}

fn note(entry: &DirEntry) -> Result<Option<Note>, NoteError> {
    let path = entry.path();
    let text = std::fs::read_to_string(&path).map_err(|error| unreadable(&path, &error))?;
    let Some(bound) = parse(&text) else {
        return Ok(None);
    };

    Ok(Some(Note {
        roadmap: bound.roadmap,
        topic: bound.topic,
        stamp: stamp(entry)?,
        path,
        body: bound.body,
    }))
}

pub fn stamp(entry: &DirEntry) -> Result<Stamp, NoteError> {
    let data = entry
        .metadata()
        .map_err(|error| unreadable(&entry.path(), &error))?;
    let modified = data
        .modified()
        .map_err(|error| unreadable(&entry.path(), &error))?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    Ok(Stamp {
        modified_nanos: modified.as_nanos(),
        size: data.len(),
    })
}

fn unreadable(path: &Path, error: &std::io::Error) -> NoteError {
    NoteError::Unreadable {
        path: path.display().to_string(),
        reason: error.to_string(),
    }
}
