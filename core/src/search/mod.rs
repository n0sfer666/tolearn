mod collect;
mod error;
mod parse;
mod query;
mod render;
mod types;

pub use collect::roadmap_id;
pub use error::SearchError;
pub use types::{Hit, Kind, Refresh};

use std::io::ErrorKind;
use std::path::Path;

use crate::atomic;
use types::Source;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Index {
    sources: Vec<Source>,
}

impl Index {
    pub fn read(path: &Path) -> Result<Self, SearchError> {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => {
                return Err(SearchError::Unreadable {
                    path: path.display().to_string(),
                    reason: error.to_string(),
                });
            }
        };
        let sources = parse::sources(&source).map_err(|error| SearchError::Malformed {
            path: path.display().to_string(),
            error,
        })?;
        Ok(Self { sources })
    }

    pub fn save(&self, path: &Path) -> Result<(), SearchError> {
        atomic::write(path, &render::text(&self.sources)).map_err(|error| SearchError::Unwritable {
            path: path.display().to_string(),
            reason: error.to_string(),
        })
    }

    pub fn refresh(&mut self, bundle: &Path, notes: &Path) -> Result<Refresh, SearchError> {
        let (roadmap, wanted) = collect::plan(bundle, notes)?;
        let mut report = Refresh::default();
        let mut fresh = Vec::with_capacity(wanted.len());

        for want in &wanted {
            match self.take(&want.path) {
                Some(source) if source.stamp == want.stamp => {
                    report.kept += 1;
                    fresh.push(source);
                }
                _ => {
                    report.indexed += 1;
                    fresh.push(collect::source(want, &roadmap)?);
                }
            }
        }

        report.dropped = self.sources.len();
        fresh.sort_by(|left, right| left.path.cmp(&right.path));
        self.sources = fresh;
        Ok(report)
    }

    pub fn find(&self, query: &str, limit: usize) -> Vec<Hit> {
        query::hits(&self.sources, query, limit)
    }

    fn take(&mut self, path: &Path) -> Option<Source> {
        let at = self.sources.iter().position(|source| source.path == path)?;
        Some(self.sources.remove(at))
    }
}
