pub mod codes;
pub mod corpus;
pub mod fields;
pub mod instance;
pub mod paths;
pub mod yaml;

use jsonschema::Validator;
use serde_json::Value;

pub use codes::{codes, declared_code};
pub use corpus::{document, fixtures, valid_documents};

use crate::repo::read;

pub const KINDS: [&str; 3] = ["roadmap", "topic", "progress"];

pub fn schema(kind: &str) -> Value {
    let path = format!("docs/schemas/{kind}.schema.json");
    serde_json::from_str(&read(&path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

pub fn validator(kind: &str) -> Validator {
    let schema = schema(kind);
    jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .unwrap_or_else(|e| panic!("{kind}.schema.json: {e}"))
}
