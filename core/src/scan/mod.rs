mod error;
mod types;

pub use error::ScanError;
pub use types::{Absent, Broken, Scan};

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::progress::Format;
use crate::roadmap::{Roadmap, parse as roadmap};
use crate::topic::parse as topic;

pub fn scan(root: &Path) -> Result<Scan, ScanError> {
    let (format, manifest) = manifest(root)?;
    let map: Roadmap = parse(&manifest, roadmap)?;

    let generated: BTreeSet<u32> = map
        .stages
        .iter()
        .filter(|stage| stage.generated)
        .map(|stage| stage.n)
        .collect();
    let mut topics = Vec::new();
    let mut absent = Vec::new();
    let mut broken = Vec::new();

    for entry in &map.topics {
        let path = root.join(&entry.file);
        if !path.exists() {
            absent.push(Absent {
                id: entry.id.clone(),
                stage: entry.stage,
                generated: generated.contains(&entry.stage),
                file: entry.file.clone(),
            });
            continue;
        }
        match parse(&path, topic) {
            Ok(document) => topics.push(document),
            Err(error) => broken.push(Broken {
                id: entry.id.clone(),
                file: entry.file.clone(),
                error,
            }),
        }
    }

    Ok(Scan {
        root: root.to_owned(),
        format,
        roadmap: map,
        topics,
        absent,
        broken,
    })
}

fn manifest(root: &Path) -> Result<(Format, PathBuf), ScanError> {
    let yaml = root.join("roadmap.yaml");
    let json = root.join("roadmap.json");
    match (yaml.exists(), json.exists()) {
        (true, true) => Err(ScanError::AmbiguousFormat {
            root: root.to_owned(),
        }),
        (true, false) => Ok((Format::Yaml, yaml)),
        (false, true) => Ok((Format::Json, json)),
        (false, false) => Err(ScanError::NoRoadmap {
            root: root.to_owned(),
        }),
    }
}

fn parse<T>(
    path: &Path,
    read: impl Fn(&str) -> Result<T, crate::yaml::ParseError>,
) -> Result<T, ScanError> {
    let source = std::fs::read_to_string(path).map_err(|error| ScanError::Unreadable {
        path: path.to_owned(),
        kind: error.kind(),
    })?;
    read(&source).map_err(|error| ScanError::Malformed {
        path: path.to_owned(),
        error,
    })
}
