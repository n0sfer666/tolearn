use std::collections::BTreeSet;

use serde_json::Value;

use crate::schema::fields::{ROLES, rows};
use crate::schema::paths::{described_fields, objects};
use crate::schema::{KINDS, schema};

#[test]
fn every_object_of_every_schema_rejects_unknown_fields() {
    for kind in KINDS {
        let schema = schema(kind);
        for (path, object) in objects(&schema) {
            let label = if path.is_empty() { "<root>" } else { &path };
            let dictionary = object.get("propertyNames").is_some()
                && object.get("properties").is_none()
                && object
                    .get("additionalProperties")
                    .is_some_and(Value::is_object);
            let closed =
                object.get("additionalProperties") == Some(&Value::Bool(false)) || dictionary;
            assert!(
                closed,
                "{kind}.schema.json: `{label}` accepts unknown fields, \
                 so «the schema describes every field» stops being provable"
            );
        }
    }
}

#[test]
fn every_described_field_is_classified_in_the_field_list() {
    let rows = rows();
    for row in &rows {
        assert!(
            ROLES.contains(&row.role.as_str()),
            "docs/fields.md: `{}` has role `{}`, expected one of {ROLES:?}",
            row.path,
            row.role
        );
    }
    let empty: Vec<&String> = Vec::new();
    for kind in KINDS {
        let described = described_fields(&schema(kind));
        let classified: BTreeSet<String> = rows
            .iter()
            .filter(|row| row.kind == kind)
            .map(|row| row.path.clone())
            .collect();
        assert_eq!(
            described.difference(&classified).collect::<Vec<_>>(),
            empty,
            "{kind}: described by the schema, missing from docs/fields.md"
        );
        assert_eq!(
            classified.difference(&described).collect::<Vec<_>>(),
            empty,
            "{kind}: classified in docs/fields.md, absent from the schema"
        );
    }
}
