use jsonschema::Validator;
use serde_json::Value;

pub fn declared_code(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or_default();
    name.split("__")
        .next()
        .filter(|code| !code.is_empty() && *code != name)
        .unwrap_or_else(|| panic!("{path}: a broken fixture is named `<code>__<what>.yaml`"))
        .to_owned()
}

pub fn codes(validator: &Validator, instance: &Value) -> Vec<String> {
    validator
        .iter_errors(instance)
        .map(|error| keyword(&error.schema_path().to_string()))
        .collect()
}

fn keyword(schema_path: &str) -> String {
    schema_path
        .rsplit('/')
        .find(|segment| !segment.is_empty() && !segment.chars().all(char::is_numeric))
        .unwrap_or_else(|| panic!("no keyword in schema path `{schema_path}`"))
        .to_owned()
}
