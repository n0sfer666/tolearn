use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::error::SearchError;
use super::types::{Document, Kind, Source};
use crate::notes::Stamp;
use crate::roadmap::{Roadmap, parse as roadmap};
use crate::topic::{Topic, parse as topic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Topic,
    Note,
}

#[derive(Debug, Clone)]
pub struct Wanted {
    pub path: PathBuf,
    pub origin: Origin,
    pub stamp: Stamp,
}

pub fn plan(bundle: &Path, notes: &Path) -> Result<(String, Vec<Wanted>), SearchError> {
    let map = manifest(bundle)?;
    let mut wanted = Vec::new();
    for entry in &map.topics {
        let path = bundle.join(&entry.file);
        if let Some(stamp) = stamp(&path)? {
            wanted.push(Wanted {
                path,
                origin: Origin::Topic,
                stamp,
            });
        }
    }
    walk(notes, &mut wanted)?;
    Ok((map.id, wanted))
}

pub fn roadmap_id(bundle: &Path) -> Result<String, SearchError> {
    Ok(manifest(bundle)?.id)
}

pub fn source(wanted: &Wanted, roadmap: &str) -> Result<Source, SearchError> {
    let text = read(&wanted.path)?;
    let (owner, documents) = match wanted.origin {
        Origin::Topic => (roadmap.to_owned(), from_topic(&text)),
        Origin::Note => from_note(&text, roadmap),
    };
    Ok(Source {
        path: wanted.path.clone(),
        roadmap: owner,
        stamp: wanted.stamp,
        documents,
    })
}

fn manifest(bundle: &Path) -> Result<Roadmap, SearchError> {
    let (_, path) = crate::scan::manifest(bundle).map_err(SearchError::Bundle)?;
    let source = read(&path)?;
    roadmap(&source)
        .map_err(|error| SearchError::Bundle(crate::scan::ScanError::Malformed { path, error }))
}

fn from_topic(text: &str) -> Vec<Document> {
    let Ok(document) = topic(text) else {
        return Vec::new();
    };
    let mut out = vec![Document {
        kind: Kind::Topic,
        topic: document.id.clone(),
        title: document.title.clone(),
        text: about(&document),
    }];
    out.extend(document.materials.iter().map(|material| Document {
        kind: Kind::Material,
        topic: document.id.clone(),
        title: material.title.clone(),
        text: format!("{}\n{}", material.url, material.note),
    }));
    out
}

fn about(document: &Topic) -> String {
    let mut lines = document.outcomes.clone();
    lines.extend(document.misconceptions.iter().cloned());
    lines.extend(document.version_context.iter().cloned());
    lines.push(document.practice.task.clone());
    lines.push(document.exam.focus.clone());
    lines.join("\n")
}

fn from_note(text: &str, roadmap: &str) -> (String, Vec<Document>) {
    let Some(bound) = crate::notes::frontmatter::parse(text) else {
        return (String::new(), Vec::new());
    };
    if bound.roadmap != roadmap {
        return (String::new(), Vec::new());
    }
    let documents = vec![Document {
        kind: Kind::Note,
        topic: bound.topic.clone(),
        title: bound.topic,
        text: bound.body,
    }];
    (roadmap.to_owned(), documents)
}

fn walk(directory: &Path, wanted: &mut Vec<Wanted>) -> Result<(), SearchError> {
    let listing = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(unreadable(directory, &error)),
    };

    for entry in listing {
        let path = entry.map_err(|error| unreadable(directory, &error))?.path();
        if path.is_dir() {
            walk(&path, wanted)?;
            continue;
        }
        if !path.extension().is_some_and(|kind| kind == "md") {
            continue;
        }
        if let Some(stamp) = stamp(&path)? {
            wanted.push(Wanted {
                path,
                origin: Origin::Note,
                stamp,
            });
        }
    }
    Ok(())
}

fn stamp(path: &Path) -> Result<Option<Stamp>, SearchError> {
    let data = match std::fs::metadata(path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(unreadable(path, &error)),
    };
    let modified = data
        .modified()
        .map_err(|error| unreadable(path, &error))?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    Ok(Some(Stamp {
        modified_nanos: modified.as_nanos(),
        size: data.len(),
    }))
}

fn read(path: &Path) -> Result<String, SearchError> {
    std::fs::read_to_string(path).map_err(|error| unreadable(path, &error))
}

fn unreadable(path: &Path, error: &std::io::Error) -> SearchError {
    SearchError::Unreadable {
        path: path.display().to_string(),
        reason: error.to_string(),
    }
}
