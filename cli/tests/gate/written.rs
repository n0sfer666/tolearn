use tolearn_core::Date;
use tolearn_core::progress::{Document, Format, Status};
use tolearn_core::status::manual;
use tolearn_core::topic::parse as topic;

use crate::repo::read;
use crate::schema::{validator, yaml::load};

const PROGRESS: &str = "fixtures/valid/progress/corpus-program.yaml";
const TOPIC: &str = "fixtures/valid/topic/stale-knowledge.yaml";
const EVERY_STATUS: [Status; 8] = [
    Status::Todo,
    Status::InProgress,
    Status::ExamPending,
    Status::Passed,
    Status::PassedOut,
    Status::StalePassed,
    Status::Blocked,
    Status::Failed,
];

#[test]
fn every_mark_the_application_writes_still_validates() {
    let validator = validator("progress");
    let topic = topic(&read(TOPIC)).unwrap();
    let today = Date::parse("2026-07-27").unwrap();

    for status in EVERY_STATUS {
        let mut document = Document::read(&read(PROGRESS), Format::Yaml).unwrap();
        let mut mark = manual(status, "2026-07-27T10:00:00Z", today, &topic);
        mark.note = Some("Отмечено руками.\n".to_owned());
        document.mark("stale-knowledge", &mark).unwrap();

        let instance = load(document.text(), "the marked progress");
        let complaints: Vec<String> = validator
            .iter_errors(&instance)
            .map(|error| format!("  {}: {error}", error.instance_path()))
            .collect();
        assert!(
            complaints.is_empty(),
            "a mark of `{status:?}` is expected to stay valid:\n{}",
            complaints.join("\n")
        );
    }
}
