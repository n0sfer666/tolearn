use crate::merge::Part;
use crate::roadmap::Roadmap;
use crate::topic::Topic;

#[derive(Debug, Clone, Copy)]
pub struct Snapshot<'a> {
    pub roadmap: &'a Roadmap,
    pub topics: &'a [Topic],
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub rewritten: Vec<Rewritten>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rewritten {
    pub id: String,
    pub title: String,
    pub changed: Vec<Part>,
    pub demoted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub n: u32,
    pub saved_at: String,
    pub bytes: u64,
}
