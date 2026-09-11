use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub modified_nanos: u128,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Topic,
    Material,
}

pub(super) const KINDS: [(&str, Kind); 2] = [("topic", Kind::Topic), ("material", Kind::Material)];

impl Kind {
    pub fn label(self) -> &'static str {
        KINDS
            .iter()
            .find(|(_, kind)| *kind == self)
            .map_or("topic", |(name, _)| name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub kind: Kind,
    pub topic: String,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub path: PathBuf,
    pub roadmap: String,
    pub stamp: Stamp,
    pub documents: Vec<Document>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub kind: Kind,
    pub roadmap: String,
    pub topic: String,
    pub title: String,
    pub snippet: String,
    pub score: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Refresh {
    pub indexed: usize,
    pub kept: usize,
    pub dropped: usize,
}
