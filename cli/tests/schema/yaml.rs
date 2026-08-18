use saphyr::{LoadableYamlNode, Scalar, Yaml};
use serde_json::{Map, Number, Value};

pub fn load(source: &str, origin: &str) -> Value {
    let documents = Yaml::load_from_str(source).unwrap_or_else(|e| panic!("{origin}: {e}"));
    assert_eq!(
        documents.len(),
        1,
        "{origin}: bundle files hold exactly one YAML document"
    );
    convert(&documents[0], origin)
}

fn convert(node: &Yaml<'_>, origin: &str) -> Value {
    match node {
        Yaml::Value(scalar) => scalar_to_json(scalar, origin),
        Yaml::Sequence(items) => {
            Value::Array(items.iter().map(|item| convert(item, origin)).collect())
        }
        Yaml::Mapping(entries) => {
            let mut map = Map::new();
            for (key, value) in entries {
                map.insert(mapping_key(key, origin), convert(value, origin));
            }
            Value::Object(map)
        }
        other => panic!("{origin}: node not representable as JSON: {other:?}"),
    }
}

fn scalar_to_json(scalar: &Scalar<'_>, origin: &str) -> Value {
    match scalar {
        Scalar::Null => Value::Null,
        Scalar::Boolean(value) => Value::Bool(*value),
        Scalar::Integer(value) => Value::Number(Number::from(*value)),
        Scalar::FloatingPoint(value) => Number::from_f64(value.into_inner())
            .map(Value::Number)
            .unwrap_or_else(|| panic!("{origin}: floating point {value} is not finite")),
        Scalar::String(value) => Value::String(value.to_string()),
    }
}

fn mapping_key(key: &Yaml<'_>, origin: &str) -> String {
    match key {
        Yaml::Value(Scalar::String(text)) => text.to_string(),
        other => panic!("{origin}: mapping key is not a string: {other:?}"),
    }
}
