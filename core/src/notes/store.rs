use std::path::{Path, PathBuf};

use super::error::NoteError;
use super::frontmatter::render;
use super::index::index;
use super::types::{Note, Stamp};

pub fn read(root: &Path, roadmap: &str, topic: &str) -> Result<Option<Note>, NoteError> {
    Ok(index(root)?
        .into_iter()
        .find(|note| note.roadmap == roadmap && note.topic == topic))
}

pub fn save(
    root: &Path,
    roadmap: &str,
    topic: &str,
    body: &str,
    expected: Option<Stamp>,
) -> Result<Stamp, NoteError> {
    let existing = read(root, roadmap, topic)?;
    if let (Some(note), Some(stamp)) = (&existing, expected)
        && note.stamp != stamp
    {
        return Err(NoteError::Conflict {
            theirs: note.body.clone(),
            ours: body.to_owned(),
        });
    }

    let path = existing
        .map(|note| note.path)
        .unwrap_or_else(|| fresh(root, roadmap, topic));
    write(&path, &render(roadmap, topic, body))?;
    written(&path)
}

fn fresh(root: &Path, roadmap: &str, topic: &str) -> PathBuf {
    root.join(roadmap).join(format!("{topic}.md"))
}

fn write(path: &Path, text: &str) -> Result<(), NoteError> {
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory).map_err(|error| unwritable(path, &error))?;
    }
    crate::atomic::write(path, text).map_err(|error| unwritable(path, &error))
}

fn written(path: &Path) -> Result<Stamp, NoteError> {
    let data = std::fs::metadata(path).map_err(|error| NoteError::Unreadable {
        path: path.display().to_string(),
        reason: error.to_string(),
    })?;
    let modified = data
        .modified()
        .map_err(|error| NoteError::Unreadable {
            path: path.display().to_string(),
            reason: error.to_string(),
        })?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    Ok(Stamp {
        modified_nanos: modified.as_nanos(),
        size: data.len(),
    })
}

fn unwritable(path: &Path, error: &std::io::Error) -> NoteError {
    NoteError::Unwritable {
        path: path.display().to_string(),
        reason: error.to_string(),
    }
}
