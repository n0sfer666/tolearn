#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "graph gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::graph::{Graph, Node, graph};
use tolearn_core::status::effective;

use support::bundles;

const TODAY: &str = "2026-07-27";

fn day(text: &str) -> Date {
    Date::parse(text).unwrap_or_else(|| panic!("`{text}` — не дата"))
}

fn drawn(passed: &[&str]) -> Graph {
    let (map, topics) = bundles::reference();
    let pairs: Vec<(&str, &str)> = passed.iter().map(|id| (*id, "passed")).collect();
    let state = bundles::recorded(&pairs);
    let statuses = effective(&map, &topics, &state, day(TODAY));
    graph(&map, &topics, &statuses)
}

fn node<'a>(drawn: &'a Graph, id: &str) -> &'a Node {
    drawn
        .nodes
        .iter()
        .find(|node| node.id == id)
        .unwrap_or_else(|| panic!("`{id}` нет в графе"))
}

#[test]
fn слой_считается_от_самой_длинной_цепочки_зависимостей() {
    let drawn = drawn(&[]);

    assert_eq!(node(&drawn, "local-runtime").layer, 0);
    assert_eq!(node(&drawn, "openai-compatible-api").layer, 1);
    assert_eq!(node(&drawn, "tokens-context-cost").layer, 2);
    assert_eq!(node(&drawn, "model-selection").layer, 3);
    assert_eq!(node(&drawn, "cp-gateway").layer, 4);
}

#[test]
fn узлы_идут_по_слоям_а_внутри_слоя_по_роадмапу() {
    let drawn = drawn(&[]);

    let order: Vec<(usize, &str)> = drawn
        .nodes
        .iter()
        .map(|node| (node.layer, node.id.as_str()))
        .collect();

    let place = |id: &str| order.iter().position(|(_, node)| *node == id).unwrap();

    assert_eq!(order[0], (0, "local-runtime"));
    assert!(place("local-runtime") < place("openai-compatible-api"));
    assert!(place("openai-compatible-api") < place("structured-output"));
    assert!(place("tokens-context-cost") < place("model-selection"));
    assert!(
        order.windows(2).all(|pair| pair[0].0 <= pair[1].0),
        "{order:?}"
    );
}

#[test]
fn тема_знает_что_разблокирует() {
    let drawn = drawn(&[]);

    assert_eq!(
        node(&drawn, "local-runtime").unlocks,
        ["openai-compatible-api", "tokens-context-cost", "cp-gateway"]
    );
    assert!(node(&drawn, "cp-gateway").unlocks.is_empty());
}

#[test]
fn узел_называет_чем_разблокируется() {
    let drawn = drawn(&[]);

    assert_eq!(
        node(&drawn, "tokens-context-cost").blocked_by,
        ["local-runtime", "openai-compatible-api"]
    );
    assert!(node(&drawn, "local-runtime").blocked_by.is_empty());
}

#[test]
fn зачтённая_зависимость_уходит_из_ожидания_но_остаётся_связью() {
    let drawn = drawn(&["local-runtime"]);

    let waiting = node(&drawn, "tokens-context-cost");

    assert_eq!(waiting.blocked_by, ["openai-compatible-api"]);
    assert_eq!(
        waiting.depends_on,
        ["local-runtime", "openai-compatible-api"]
    );
}

#[test]
fn граф_несёт_статус_и_название_каждой_темы() {
    let drawn = drawn(&["local-runtime"]);

    let done = node(&drawn, "local-runtime");

    assert_eq!(done.status.label(), "passed");
    assert!(!done.title.is_empty(), "узел без названия");
    assert_eq!(node(&drawn, "cp-gateway").status.label(), "blocked");
}

#[test]
fn цикл_не_вешает_расчёт() {
    let (map, topics) = bundles::corpus_whole();
    let statuses = effective(&map, &topics, &bundles::recorded(&[]), day(TODAY));

    let drawn = graph(&map, &topics, &statuses);

    assert_eq!(drawn.nodes.len(), map.topics.len());
    assert_eq!(node(&drawn, "stale-knowledge").layer, 0);
    assert_eq!(node(&drawn, "question-shapes").layer, 2);
    assert_eq!(
        node(&drawn, "cycle-b").layer,
        1,
        "ребро назад в цикле посчитано за слой"
    );
    assert_eq!(
        node(&drawn, "cycle-a").layer,
        2,
        "ребро назад в цикле посчитано за слой"
    );
}

#[test]
fn зависимость_вне_роадмапа_в_граф_не_попадает() {
    let (map, mut topics) = bundles::reference();
    let place = bundles::document(&topics, "tokens-context-cost");
    topics[place].depends_on.push("ghost-topic".to_owned());
    let statuses = effective(&map, &topics, &bundles::recorded(&[]), day(TODAY));

    let drawn = graph(&map, &topics, &statuses);

    let waiting = node(&drawn, "tokens-context-cost");
    assert_eq!(
        waiting.depends_on,
        ["local-runtime", "openai-compatible-api"],
        "тема связана с тем, чего в программе нет"
    );
    assert_eq!(
        waiting.blocked_by,
        ["local-runtime", "openai-compatible-api"]
    );
    assert_eq!(waiting.layer, 2, "несуществующая связь сдвинула слой");
}

#[test]
fn внутри_слоя_темы_идут_в_порядке_роадмапа() {
    let (map, topics) = bundles::reference();
    let statuses = effective(&map, &topics, &bundles::recorded(&[]), day(TODAY));

    let drawn = graph(&map, &topics, &statuses);

    let second: Vec<&str> = drawn
        .nodes
        .iter()
        .filter(|node| node.layer == 2)
        .map(|node| node.id.as_str())
        .collect();
    let places: Vec<usize> = second.iter().map(|id| bundles::entry(&map, id)).collect();

    assert!(second.len() > 1, "слой из одной темы порядок не проверяет");
    assert!(
        places.windows(2).all(|pair| pair[0] < pair[1]),
        "внутри слоя порядок разошёлся с роадмапом: {second:?}"
    );
}

#[test]
fn несгенерированная_тема_остаётся_узлом_и_держит_связи() {
    let (map, whole) = bundles::corpus_whole();
    let topics: Vec<_> = whole
        .into_iter()
        .filter(|topic| topic.id != "stale-knowledge")
        .collect();
    let statuses = effective(&map, &topics, &bundles::recorded(&[]), day(TODAY));

    let drawn = graph(&map, &topics, &statuses);

    let absent = node(&drawn, "stale-knowledge");
    assert_eq!(absent.layer, 0, "ненаписанная тема уехала со своего слоя");
    assert!(
        absent.depends_on.is_empty(),
        "у ненаписанной темы взялись связи"
    );
    assert_eq!(absent.unlocks, ["offline-edge"]);
    assert_eq!(node(&drawn, "offline-edge").blocked_by, ["stale-knowledge"]);
}
