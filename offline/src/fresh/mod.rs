mod body;
mod verdict;
mod web;

pub use body::body_hash;
pub use verdict::{Freshness, Mark, WINDOW, freshness};
pub use web::Conditional;

use crate::store::Checked;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kept {
    pub body_hash: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Same,
    Big {
        etag: Option<String>,
        last_modified: Option<String>,
    },
    Sent {
        bytes: Vec<u8>,
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub fetch: bool,
    pub mark: Checked,
}

pub trait Probe {
    fn ask(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<Answer, String>;
}

pub fn weighed(kept: &Kept, answer: &Answer, at: i64) -> Change {
    match answer {
        Answer::Same => Change {
            fetch: false,
            mark: Checked {
                body_hash: kept.body_hash.clone(),
                etag: kept.etag.clone(),
                last_modified: kept.last_modified.clone(),
                at,
            },
        },
        Answer::Big {
            etag,
            last_modified,
        } => Change {
            fetch: true,
            mark: Checked {
                body_hash: None,
                etag: etag.clone(),
                last_modified: last_modified.clone(),
                at,
            },
        },
        Answer::Sent {
            bytes,
            etag,
            last_modified,
        } => sent(
            kept,
            body_hash(bytes),
            etag.clone(),
            last_modified.clone(),
            at,
        ),
    }
}

fn sent(
    kept: &Kept,
    hash: String,
    etag: Option<String>,
    last_modified: Option<String>,
    at: i64,
) -> Change {
    Change {
        fetch: kept.body_hash.as_deref() != Some(hash.as_str()),
        mark: Checked {
            body_hash: Some(hash),
            etag,
            last_modified,
            at,
        },
    }
}
