mod graph;
mod manifest;
mod order;
mod record;
mod scc;
mod violation;

pub use violation::Violation;

use crate::progress::Progress;
use crate::roadmap::Roadmap;
use crate::topic::Topic;

pub fn validate(roadmap: &Roadmap, topics: &[Topic]) -> Vec<Violation> {
    let mut found = Vec::new();
    manifest::check(roadmap, topics, &mut found);
    graph::check(roadmap, topics, &mut found);
    found
}

pub fn ordered(roadmap: &Roadmap, topics: &[Topic]) -> Vec<Violation> {
    let mut found = Vec::new();
    order::check(roadmap, topics, &mut found);
    found
}

pub fn tracked(roadmap: &Roadmap, progress: &Progress) -> Vec<Violation> {
    let mut found = Vec::new();
    record::check(roadmap, progress, &mut found);
    found
}
