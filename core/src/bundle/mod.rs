mod graph;
mod manifest;
mod scc;
mod violation;

pub use violation::Violation;

use crate::roadmap::Roadmap;
use crate::topic::Topic;

pub fn validate(roadmap: &Roadmap, topics: &[Topic]) -> Vec<Violation> {
    let mut found = Vec::new();
    manifest::check(roadmap, topics, &mut found);
    graph::check(roadmap, topics, &mut found);
    found
}
