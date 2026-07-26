use std::collections::BTreeSet;

use crate::schema::{KINDS, codes, declared_code, document, fixtures, validator};

const MINIMUM_CODES: usize = 12;

#[test]
fn every_broken_fixture_is_rejected_with_the_code_in_its_name() {
    for kind in KINDS {
        let validator = validator(kind);
        for path in fixtures("broken", kind) {
            let expected = declared_code(&path);
            let observed = codes(&validator, &document(&path));
            assert_eq!(
                observed,
                vec![expected.clone()],
                "{path} is expected to be rejected by `{expected}` once and by nothing else"
            );
        }
    }
}

#[test]
fn the_broken_corpus_covers_distinct_codes() {
    let mut codes: BTreeSet<String> = BTreeSet::new();
    for kind in KINDS {
        for path in fixtures("broken", kind) {
            codes.insert(declared_code(&path));
        }
    }
    assert!(
        codes.len() >= MINIMUM_CODES,
        "broken corpus rejects on {} distinct codes, {MINIMUM_CODES} is the floor: {codes:?}",
        codes.len()
    );
}
