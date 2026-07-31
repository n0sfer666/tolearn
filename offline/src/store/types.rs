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
    pub body_hash: Option<String>,
    pub checked_at: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Checked {
    pub body_hash: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub at: i64,
}
