use std::collections::HashMap;

use super::violation::Violation;
use crate::roadmap::Roadmap;
use crate::topic::Topic;

pub fn check(roadmap: &Roadmap, topics: &[Topic], found: &mut Vec<Violation>) {
    let place: HashMap<&str, usize> = roadmap
        .topics
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry.id.as_str(), index))
        .collect();

    for topic in topics {
        let Some(&index) = place.get(topic.id.as_str()) else {
            continue;
        };
        for dependency in &topic.depends_on {
            if place
                .get(dependency.as_str())
                .is_some_and(|&at| at >= index)
            {
                found.push(Violation::ForwardDependency {
                    topic: topic.id.clone(),
                    depends_on: dependency.clone(),
                });
            }
        }
    }
}
