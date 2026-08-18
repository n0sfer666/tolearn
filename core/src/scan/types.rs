use std::path::PathBuf;

use super::error::ScanError;
use crate::progress::Format;
use crate::roadmap::Roadmap;
use crate::topic::Topic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scan {
    pub root: PathBuf,
    pub format: Format,
    pub roadmap: Roadmap,
    pub topics: Vec<Topic>,
    pub absent: Vec<Absent>,
    pub broken: Vec<Broken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Absent {
    pub id: String,
    pub stage: u32,
    pub generated: bool,
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Broken {
    pub id: String,
    pub file: String,
    pub error: ScanError,
}
