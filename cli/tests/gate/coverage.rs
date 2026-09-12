use crate::schema::instance::{observed_by_kind, values_at};
use crate::schema::paths::fixed_values;
use crate::schema::{KINDS, schema};

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
