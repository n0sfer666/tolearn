use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::error::SearchError;
use super::types::{Document, Kind, Source};
use crate::notes::Stamp;
use crate::roadmap::{Roadmap, parse as roadmap};
use crate::topic::{Topic, parse as topic};

#[derive(Debug, Clone)]
pub struct Wanted {
    pub path: PathBuf,
    pub stamp: Stamp,
}

pub fn plan(bundle: &Path) -> Result<(String, Vec<Wanted>), SearchError> {
    let map = manifest(bundle)?;
    let wanted = topics(bundle, &map)?;
    Ok((map.id, wanted))
}

fn topics(bundle: &Path, map: &Roadmap) -> Result<Vec<Wanted>, SearchError> {
    let mut wanted = Vec::new();
    for entry in &map.topics {
        let path = bundle.join(&entry.file);
        if let Some(stamp) = stamp(&path)? {
            wanted.push(Wanted { path, stamp });
        }
    }
    Ok(wanted)
}

pub fn roadmap_id(bundle: &Path) -> Result<String, SearchError> {
    Ok(manifest(bundle)?.id)
}

pub fn source(wanted: &Wanted, roadmap: &str) -> Result<Source, SearchError> {
    let text = read(&wanted.path)?;
    Ok(Source {
        path: wanted.path.clone(),
        roadmap: roadmap.to_owned(),
        stamp: wanted.stamp,
        documents: from_topic(&text),
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
