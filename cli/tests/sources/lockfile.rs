use std::collections::{BTreeMap, BTreeSet};

use super::parse;

pub type Graph = BTreeMap<String, BTreeSet<String>>;

pub fn locked_graph() -> Graph {
    let lock = parse("Cargo.lock");
    let packages = lock["package"].as_array().unwrap();
    let ids: BTreeSet<String> = packages.iter().map(id).collect();
    packages
        .iter()
        .map(|package| (id(package), edges(package, &ids)))
        .collect()
}

pub fn reachable_from(root: &str, graph: &Graph) -> BTreeSet<String> {
    let mut queue: Vec<String> = graph.keys().filter(|id| named(id, root)).cloned().collect();
    assert!(!queue.is_empty(), "{root} is not in Cargo.lock");

    let mut seen = BTreeSet::new();
    while let Some(package) = queue.pop() {
        let Some(edges) = graph.get(&package) else {
            continue;
        };
        for edge in edges {
            if seen.insert(edge.clone()) {
                queue.push(edge.clone());
            }
        }
    }
    seen.iter().map(|id| name_of(id).to_owned()).collect()
}

fn id(package: &toml::Value) -> String {
    let name = package["name"].as_str().unwrap();
    let version = package["version"].as_str().unwrap();
    format!("{name} {version}")
}

fn edges(package: &toml::Value, ids: &BTreeSet<String>) -> BTreeSet<String> {
    let Some(edges) = package.get("dependencies").and_then(toml::Value::as_array) else {
        return BTreeSet::new();
    };
    edges
        .iter()
        .filter_map(toml::Value::as_str)
        .flat_map(|edge| resolve(edge, ids))
        .collect()
}

fn resolve(edge: &str, ids: &BTreeSet<String>) -> Vec<String> {
    let mut parts = edge.split_whitespace();
    let name = parts
        .next()
        .unwrap_or_else(|| panic!("Cargo.lock: empty dependency"));
    if let Some(version) = parts.next() {
        let id = format!("{name} {version}");
        assert!(ids.contains(&id), "Cargo.lock: {id} has no [[package]]");
        return vec![id];
    }
    let matching: Vec<String> = ids.iter().filter(|id| named(id, name)).cloned().collect();
    assert!(
        !matching.is_empty(),
        "Cargo.lock: {name} has no [[package]]"
    );
    matching
}

fn named(id: &str, name: &str) -> bool {
    name_of(id) == name
}

fn name_of(id: &str) -> &str {
    id.split_whitespace()
        .next()
        .unwrap_or_else(|| panic!("Cargo.lock: empty package id"))
}
