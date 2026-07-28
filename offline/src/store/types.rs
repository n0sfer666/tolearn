use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched<'a> {
    pub kind: &'a str,
    pub bytes: &'a [u8],
    pub etag: Option<&'a str>,
    pub last_modified: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    pub hash: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    pub hash: String,
    pub path: PathBuf,
    pub kind: String,
    pub size: u64,
    pub fetched_at: i64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}
