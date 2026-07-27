use std::collections::{BTreeSet, HashMap, VecDeque};

use super::scc;
use super::violation::Violation;
use crate::roadmap::Roadmap;
use crate::topic::Topic;

pub fn check(roadmap: &Roadmap, topics: &[Topic], found: &mut Vec<Violation>) {
    let graph = build(roadmap, topics, found);
    for component in scc::components(&graph.edges) {
        if let Some(chain) = graph.cycle_through(&component) {
            found.push(Violation::DependencyCycle { chain });
        }
    }
}

struct Graph {
    order: Vec<String>,
    edges: Vec<Vec<usize>>,
}

fn build(roadmap: &Roadmap, topics: &[Topic], found: &mut Vec<Violation>) -> Graph {
    let declared: BTreeSet<&str> = roadmap
        .topics
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    let documents: HashMap<&str, &Topic> = topics
        .iter()
        .map(|topic| (topic.id.as_str(), topic))
        .collect();

    let order: Vec<String> = roadmap
        .topics
        .iter()
        .filter(|entry| documents.contains_key(entry.id.as_str()))
        .map(|entry| entry.id.clone())
        .collect();
    let place: HashMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect();

    let mut edges = vec![Vec::new(); order.len()];
    for (index, id) in order.iter().enumerate() {
        for dependency in &documents[id.as_str()].depends_on {
            if !declared.contains(dependency.as_str()) {
                found.push(Violation::UnknownDependency {
                    topic: id.clone(),
                    depends_on: dependency.clone(),
                });
            } else if let Some(&target) = place.get(dependency.as_str()) {
                edges[index].push(target);
            }
        }
    }

    Graph { order, edges }
}

impl Graph {
    fn cycle_through(&self, component: &BTreeSet<usize>) -> Option<Vec<String>> {
        let &start = component.iter().next()?;
        let mut parent = HashMap::new();
        let mut queue = VecDeque::from([start]);
        let mut seen = BTreeSet::from([start]);
        while let Some(node) = queue.pop_front() {
            for &next in &self.edges[node] {
                if !component.contains(&next) {
                    continue;
                }
                if next == start {
                    return Some(self.trace(start, node, &parent));
                }
                if seen.insert(next) {
                    parent.insert(next, node);
                    queue.push_back(next);
                }
            }
        }
        None
    }

    fn trace(&self, start: usize, end: usize, parent: &HashMap<usize, usize>) -> Vec<String> {
        let mut chain = vec![end];
        let mut at = end;
        while at != start {
            at = parent[&at];
            chain.push(at);
        }
        chain.reverse();
        chain
            .into_iter()
            .map(|node| self.order[node].clone())
            .collect()
    }
}
