#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub modified_nanos: u128,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub name: String,
    pub stamp: Stamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Stage,
    Block,
}

pub(super) const KINDS: [(&str, Kind); 2] = [("stage", Kind::Stage), ("block", Kind::Block)];

impl Kind {
    pub fn label(self) -> &'static str {
        KINDS
            .iter()
            .find(|(_, kind)| *kind == self)
            .map_or("block", |(name, _)| name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub kind: Kind,
    pub node: String,
    pub node_title: String,
    pub stage: String,
    pub title: String,
    pub block: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub program: String,
    pub files: Vec<Seen>,
    pub documents: Vec<Document>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub kind: Kind,
    pub program: String,
    pub node: String,
    pub node_title: String,
    pub stage: String,
    pub title: String,
    pub block: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Refresh {
    pub indexed: usize,
    pub kept: usize,
    pub dropped: usize,
}
