use crate::schema::{KINDS, valid_documents, validator};

#[test]
fn every_valid_document_validates() {
    for kind in KINDS {
        let validator = validator(kind);
        for (path, instance) in valid_documents(kind) {
            let complaints: Vec<String> = validator
                .iter_errors(&instance)
                .map(|error| format!("  {}: {error}", error.instance_path()))
                .collect();
            assert!(
                complaints.is_empty(),
                "{path} is expected to be valid:\n{}",
                complaints.join("\n")
            );
        }
    }
}
