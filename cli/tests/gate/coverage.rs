use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::schema::instance::{attempts, observed_by_kind, topic_ids, values_at};
use crate::schema::paths::fixed_values;
use crate::schema::{KINDS, schema, valid_documents};

type Graph = BTreeMap<String, BTreeSet<String>>;

#[test]
fn every_fixed_value_appears_in_the_valid_corpus() {
    let observed = observed_by_kind();
    for kind in KINDS {
        let seen = &observed[kind];
        for (path, values) in fixed_values(&schema(kind)) {
            let corpus = values_at(seen, &path);
            let missing: Vec<&String> = values.difference(&corpus).collect();
            assert!(
                missing.is_empty(),
                "{kind}: `{path}` fixes {missing:?}, and no valid document carries them"
            );
        }
    }
}

#[test]
fn the_valid_corpus_carries_attempt_history() {
    let recorded: Vec<Value> = valid_documents("progress")
        .iter()
        .flat_map(|(_, progress)| attempts(progress))
        .collect();
    let sources: BTreeSet<&str> = recorded
        .iter()
        .filter_map(|attempt| attempt["source"].as_str())
        .collect();
    assert!(
        sources.contains("exam") && sources.contains("manual"),
        "valid progress fixtures carry attempts of both sources, found {sources:?}"
    );
    assert!(
        recorded
            .iter()
            .all(|attempt| attempt.get("raw").is_some_and(Value::is_string)),
        "every recorded attempt keeps `raw` verbatim"
    );
}

#[test]
fn the_valid_corpus_carries_a_dependency_cycle() {
    let graph: Graph = topics_by_id()
        .into_iter()
        .map(|(id, topic)| (id, listed(&topic["depends_on"])))
        .collect();
    let cyclic = valid_documents("roadmap")
        .iter()
        .any(|(_, roadmap)| closes_a_cycle(&graph, &topic_ids(roadmap)));
    assert!(
        cyclic,
        "no valid program closes a dependency cycle for the integrity validator to catch"
    );
}

#[test]
fn the_valid_corpus_diverges_from_the_roadmap_defaults() {
    let topics = topics_by_id();
    let divergent = valid_documents("roadmap").iter().any(|(_, roadmap)| {
        let defaults = &roadmap["defaults"]["revalidate_after_days"];
        topic_ids(roadmap).iter().any(|id| {
            topics.get(id).is_some_and(|topic| {
                let by_volatility = &defaults[topic["volatility"].as_str().unwrap_or_default()];
                by_volatility.is_number() && *by_volatility != topic["revalidate_after_days"]
            })
        })
    });
    assert!(
        divergent,
        "no valid topic disagrees with the roadmap defaults on `revalidate_after_days`"
    );
}

fn topics_by_id() -> BTreeMap<String, Value> {
    let mut found: BTreeMap<String, Value> = BTreeMap::new();
    for (path, topic) in valid_documents("topic") {
        let id = topic["id"].as_str().unwrap_or_default().to_owned();
        assert!(
            found.insert(id.clone(), topic).is_none(),
            "{path}: topic id `{id}` is already taken in the valid corpus"
        );
    }
    found
}

fn listed(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned))
        .map(Iterator::collect)
        .unwrap_or_default()
}

fn closes_a_cycle(graph: &Graph, within: &[String]) -> bool {
    let scope: BTreeSet<&String> = within.iter().collect();
    within
        .iter()
        .any(|start| reaches(graph, &scope, start, start, &mut BTreeSet::new()))
}

fn reaches(
    graph: &Graph,
    scope: &BTreeSet<&String>,
    from: &str,
    target: &str,
    seen: &mut BTreeSet<String>,
) -> bool {
    graph
        .get(from)
        .into_iter()
        .flatten()
        .filter(|next| scope.contains(next))
        .any(|next| {
            next == target
                || (seen.insert(next.clone()) && reaches(graph, scope, next, target, seen))
        })
}
