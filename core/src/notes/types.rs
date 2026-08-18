use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub modified_nanos: u128,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub roadmap: String,
    pub topic: String,
    pub path: PathBuf,
    pub body: String,
    pub stamp: Stamp,
}
