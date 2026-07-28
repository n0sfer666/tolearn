mod layer;
mod types;

pub use types::{Graph, Node};

use std::collections::{BTreeSet, HashMap};

use crate::progress::Status;
use crate::roadmap::Roadmap;
use crate::status::{Statuses, is_done};
use crate::topic::Topic;

pub fn graph(roadmap: &Roadmap, topics: &[Topic], statuses: &Statuses) -> Graph {
    let order: Vec<String> = roadmap
        .topics
        .iter()
        .map(|entry| entry.id.clone())
        .collect();
    let declared: BTreeSet<&str> = order.iter().map(String::as_str).collect();
    let edges = wires(&order, topics, &declared);
    let layers = layer::layers(&order, &edges);

    let mut nodes: Vec<Node> = roadmap
        .topics
        .iter()
        .map(|entry| {
            let depends_on = edges.get(&entry.id).cloned().unwrap_or_default();
            Node {
                blocked_by: waiting(&depends_on, statuses),
                unlocks: unlocked_by(&entry.id, &order, &edges),
                layer: layers.get(&entry.id).copied().unwrap_or(0),
                status: statuses.get(&entry.id).unwrap_or(Status::Todo),
                title: entry.title.clone(),
                id: entry.id.clone(),
                depends_on,
            }
        })
        .collect();
    nodes.sort_by_key(|node| node.layer);

    Graph { nodes }
}

fn wires(
    order: &[String],
    topics: &[Topic],
    declared: &BTreeSet<&str>,
) -> HashMap<String, Vec<String>> {
    order
        .iter()
        .map(|id| {
            let named = topics
                .iter()
                .find(|topic| &topic.id == id)
                .map(|topic| {
                    topic
                        .depends_on
                        .iter()
                        .filter(|dependency| declared.contains(dependency.as_str()))
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            (id.clone(), named)
        })
        .collect()
}

fn waiting(depends_on: &[String], statuses: &Statuses) -> Vec<String> {
    depends_on
        .iter()
        .filter(|dependency| !statuses.get(dependency).is_some_and(is_done))
        .cloned()
        .collect()
}

fn unlocked_by(id: &str, order: &[String], edges: &HashMap<String, Vec<String>>) -> Vec<String> {
    order
        .iter()
        .filter(|other| {
            edges
                .get(other.as_str())
                .is_some_and(|named| named.iter().any(|dependency| dependency == id))
        })
        .cloned()
        .collect()
}
