mod collect;
mod documents;
mod error;
mod parse;
mod query;
mod render;
mod types;

pub use error::SearchError;
pub use types::{Hit, Kind, Refresh};

use std::path::Path;

use crate::atomic;
use crate::library::Library;
use types::Source;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Index {
    sources: Vec<Source>,
}

impl Index {
    pub fn read(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|source| Self::parse(&source, path).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), SearchError> {
        atomic::write(path, &self.text()).map_err(|error| SearchError::Unwritable {
            path: path.display().to_string(),
            reason: error.to_string(),
        })
    }

    pub fn refresh(&mut self, library: &Library) -> Result<Refresh, SearchError> {
        let mut report = Refresh::default();
        let mut fresh = Vec::new();
        for want in collect::plan(library)? {
            match self.take(&want.program) {
                Some(source) if source.files == want.files => {
                    report.kept += 1;
                    fresh.push(source);
                }
                stale => match documents::source(library, want) {
                    Some(source) => {
                        report.indexed += 1;
                        fresh.push(source);
                    }
                    None => report.dropped += usize::from(stale.is_some()),
                },
            }
        }
        report.dropped += self.sources.len();
        self.sources = fresh;
        Ok(report)
    }

    pub fn text(&self) -> String {
        render::text(&self.sources)
    }

    pub fn parse(source: &str, path: &Path) -> Result<Self, SearchError> {
        let sources = parse::sources(source).map_err(|error| SearchError::Malformed {
            path: path.display().to_string(),
            error,
        })?;
        Ok(Self { sources })
    }

    pub fn find(&self, query: &str, limit: usize) -> Vec<Hit> {
        query::hits(&self.sources, query, limit)
    }

    fn take(&mut self, program: &str) -> Option<Source> {
        let at = self
            .sources
            .iter()
            .position(|source| source.program == program)?;
        Some(self.sources.remove(at))
    }
}
