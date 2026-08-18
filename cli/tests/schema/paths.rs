use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

const TRAVERSED: [&str; 3] = ["properties", "items", "additionalProperties"];

const TERMINAL: [&str; 21] = [
    "$schema",
    "$id",
    "$ref",
    "$defs",
    "title",
    "description",
    "type",
    "required",
    "enum",
    "const",
    "format",
    "pattern",
    "minLength",
    "maxLength",
    "minimum",
    "minItems",
    "maxItems",
    "uniqueItems",
    "minProperties",
    "propertyNames",
    "dependentRequired",
];

pub fn described_fields(schema: &Value) -> BTreeSet<String> {
    nodes(schema)
        .into_iter()
        .map(|(path, _)| path)
        .filter(|path| is_field(path))
        .collect()
}

pub fn objects(schema: &Value) -> Vec<(String, &Value)> {
    nodes(schema)
        .into_iter()
        .filter(|(_, node)| declares_object(node) || node.get("properties").is_some())
        .collect()
}

pub fn fixed_values(schema: &Value) -> BTreeMap<String, BTreeSet<String>> {
    let mut found = BTreeMap::new();
    for (path, node) in nodes(schema) {
        let listed = node.get("enum").and_then(Value::as_array);
        let single = node.get("const").map(std::slice::from_ref);
        let Some(values) = listed.map(Vec::as_slice).or(single) else {
            continue;
        };
        let strings: BTreeSet<String> = values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        assert_eq!(
            strings.len(),
            values.len(),
            "`{path}`: `enum`/`const` holds a value the corpus gate cannot watch, \
             it only knows strings"
        );
        found.insert(path, strings);
    }
    found
}

fn nodes(schema: &Value) -> Vec<(String, &Value)> {
    let mut found = Vec::new();
    collect(schema, schema, String::new(), 0, &mut found);
    found
}

fn collect<'a>(
    node: &'a Value,
    root: &'a Value,
    path: String,
    depth: usize,
    found: &mut Vec<(String, &'a Value)>,
) {
    let node = resolve(node, root, depth, &path);
    known_keywords(node, &path);
    if let Some(properties) = node.get("properties").and_then(Value::as_object) {
        for (name, property) in properties {
            collect(property, root, join(&path, name), depth + 1, found);
        }
    }
    if let Some(items) = node.get("items") {
        collect(items, root, format!("{path}[]"), depth + 1, found);
    }
    if let Some(values) = node.get("additionalProperties").filter(|v| v.is_object()) {
        collect(values, root, join(&path, "*"), depth + 1, found);
    }
    found.push((path, node));
}

fn known_keywords(node: &Value, path: &str) {
    let label = if path.is_empty() { "<root>" } else { path };
    for keyword in node.as_object().into_iter().flat_map(serde_json::Map::keys) {
        assert!(
            TRAVERSED.contains(&keyword.as_str()) || TERMINAL.contains(&keyword.as_str()),
            "`{label}`: keyword `{keyword}` is unknown to the walker, \
             so whatever it describes is invisible to every gate built on it"
        );
    }
}

fn declares_object(node: &Value) -> bool {
    match node.get("type") {
        Some(Value::String(name)) => name == "object",
        Some(Value::Array(names)) => names.iter().any(|name| name == "object"),
        _ => false,
    }
}

fn is_field(path: &str) -> bool {
    let last = path.rsplit('.').next().unwrap_or_default();
    !path.is_empty() && last != "*" && !last.ends_with("[]")
}

fn resolve<'a>(node: &'a Value, root: &'a Value, depth: usize, path: &str) -> &'a Value {
    assert!(depth < 16, "schema nesting deeper than 16: cycle in $ref?");
    let Some(reference) = node.get("$ref").and_then(Value::as_str) else {
        return node;
    };
    assert_eq!(
        node.as_object().map(serde_json::Map::len),
        Some(1),
        "`{path}`: `$ref` carries siblings, and the walker keeps only the target"
    );
    let name = reference
        .strip_prefix("#/$defs/")
        .unwrap_or_else(|| panic!("only local `#/$defs/*` references are supported: {reference}"));
    root.get("$defs")
        .and_then(|defs| defs.get(name))
        .unwrap_or_else(|| panic!("$ref points nowhere: {reference}"))
}

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}
