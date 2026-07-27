#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "merge gate: a panic here is the report"
)]

mod support;

use tolearn_core::merge::{Part, Report, merge};
use tolearn_core::progress::{Document, Format, Mark, Status, parse};
use tolearn_core::roadmap::Roadmap;
use tolearn_core::topic::Topic;

use support::{bundles, read};

const PROGRESS: &str = "fixtures/valid/progress/corpus-program.yaml";

fn document() -> Document {
    Document::read(&read(PROGRESS), Format::Yaml).unwrap()
}

fn bundle() -> (Roadmap, Vec<Topic>) {
    bundles::corpus_whole()
}

fn topic<'a>(topics: &'a mut [Topic], id: &str) -> &'a mut Topic {
    topics
        .iter_mut()
        .find(|topic| topic.id == id)
        .unwrap_or_else(|| panic!("`{id}` is not in the bundle"))
}

fn pass(document: &mut Document, id: &str) {
    let mark = Mark {
        status: Status::Passed,
        at: "2026-07-27T10:00:00Z".to_owned(),
        passed_at: Some("2026-07-27".to_owned()),
        next_review_at: Some("2026-08-26".to_owned()),
        note: None,
    };
    document.mark(id, &mark).unwrap();
}

fn status(document: &Document, id: &str) -> Status {
    document
        .progress()
        .state(id)
        .unwrap_or_else(|| panic!("`{id}` is not in the progress"))
        .status
}

type Change = fn(&mut Topic);

fn merged(document: &mut Document, roadmap: &Roadmap, before: &[Topic], after: &[Topic]) -> Report {
    merge(document, roadmap, before, after).unwrap()
}

#[test]
fn a_bundle_that_did_not_change_changes_nothing() {
    let (roadmap, topics) = bundle();
    let mut document = document();
    let before = document.text().to_owned();

    let report = merged(&mut document, &roadmap, &topics, &topics);

    assert_eq!(document.text(), before);
    assert_eq!(report.stale, []);
    assert_eq!(report.added, [] as [String; 0]);
    assert_eq!(report.orphaned, [] as [String; 0]);
    assert!(!report.kept.is_empty());
}

#[test]
fn the_progress_of_a_topic_that_stayed_is_kept_whole() {
    let (roadmap, mut topics) = bundle();
    let mut document = document();
    let kept = document
        .progress()
        .state("stale-knowledge")
        .unwrap()
        .clone();
    topic(&mut topics, "cycle-a").title = "Переписанный заголовок".to_owned();

    let report = merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_eq!(document.progress().state("stale-knowledge"), Some(&kept));
    assert!(report.kept.contains(&"stale-knowledge".to_owned()));
}

#[test]
fn a_rewritten_exam_brings_a_pass_down_to_stale_passed() {
    let (roadmap, mut topics) = bundle();
    let mut document = document();
    pass(&mut document, "cycle-a");
    topic(&mut topics, "cycle-a").exam.focus = "Другой фокус экзамена".to_owned();

    let report = merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_eq!(status(&document, "cycle-a"), Status::StalePassed);
    assert_eq!(report.stale.len(), 1);
    assert_eq!(report.stale[0].id, "cycle-a");
    assert_eq!(report.stale[0].changed, [Part::Exam]);
}

#[test]
fn every_part_that_is_taught_or_checked_brings_a_pass_down() {
    let cases: [(Part, Change); 6] = [
        (Part::DependsOn, |topic| topic.depends_on.clear()),
        (Part::Outcomes, |topic| {
            topic.outcomes.push("Новый результат".to_owned());
        }),
        (Part::Misconceptions, |topic| {
            topic.misconceptions.push("Новое заблуждение".to_owned());
        }),
        (Part::Practice, |topic| {
            topic.practice.task = "Другая практика".to_owned();
        }),
        (Part::Questions, |topic| {
            let question = bundles::a_question();
            topic.questions.push(question);
        }),
        (Part::Exam, |topic| topic.exam.artifact_required = true),
    ];

    for (part, change) in cases {
        let (roadmap, mut topics) = bundle();
        let mut document = document();
        pass(&mut document, "cycle-a");
        change(topic(&mut topics, "cycle-a"));

        let report = merged(&mut document, &roadmap, &bundle().1, &topics);

        assert_eq!(
            status(&document, "cycle-a"),
            Status::StalePassed,
            "{part:?}"
        );
        assert_eq!(report.stale[0].changed, [part], "{part:?}");
    }
}

#[test]
fn what_is_neither_taught_nor_checked_leaves_the_pass_alone() {
    let changes: [Change; 4] = [
        |topic| topic.title = "Другой заголовок".to_owned(),
        |topic| topic.est_hours.max += 1,
        |topic| topic.materials.clear(),
        |topic| topic.verified_at = "2020-01-01".to_owned(),
    ];

    for change in changes {
        let (roadmap, mut topics) = bundle();
        let mut document = document();
        pass(&mut document, "cycle-a");
        change(topic(&mut topics, "cycle-a"));

        let report = merged(&mut document, &roadmap, &bundle().1, &topics);

        assert_eq!(status(&document, "cycle-a"), Status::Passed);
        assert_eq!(report.stale, []);
    }
}

#[test]
fn a_changed_topic_that_was_not_passed_keeps_the_status_it_had() {
    let (roadmap, mut topics) = bundle();
    let mut document = document();
    let was = status(&document, "cycle-b");
    topic(&mut topics, "cycle-b").exam.focus = "Другой фокус".to_owned();

    let report = merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_ne!(was, Status::Passed);
    assert_eq!(status(&document, "cycle-b"), was);
    assert_eq!(report.stale, []);
}

#[test]
fn a_topic_the_generator_added_starts_as_todo() {
    let (mut roadmap, mut topics) = bundle();
    let mut document = document();
    let mut fresh = topic(&mut topics, "cycle-a").clone();
    fresh.id = "fresh-topic".to_owned();
    let mut entry = roadmap.topics[0].clone();
    entry.id = "fresh-topic".to_owned();
    roadmap.topics.push(entry);
    topics.push(fresh);

    let report = merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_eq!(status(&document, "fresh-topic"), Status::Todo);
    assert_eq!(report.added, ["fresh-topic"]);
    let state = document.progress().state("fresh-topic").unwrap();
    assert!(state.attempts.is_empty());
    assert_eq!(state.passed_at, None);
    assert_eq!(state.next_review_at, None);
}

#[test]
fn a_topic_that_disappeared_is_orphaned_rather_than_deleted() {
    let (mut roadmap, mut topics) = bundle();
    let mut document = document();
    let kept = document.progress().state("cycle-a").unwrap().clone();
    roadmap.topics.retain(|entry| entry.id != "cycle-a");
    topics.retain(|topic| topic.id != "cycle-a");

    let report = merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_eq!(document.progress().state("cycle-a"), Some(&kept));
    assert_eq!(report.orphaned, ["cycle-a"]);
    assert!(!report.kept.contains(&"cycle-a".to_owned()));
}

#[test]
fn a_topic_the_generator_has_not_written_yet_is_not_an_orphan() {
    let (roadmap, mut topics) = bundle();
    let mut document = document();
    topics.retain(|topic| topic.id != "cycle-a");
    pass(&mut document, "cycle-a");

    let report = merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_eq!(report.orphaned, [] as [String; 0]);
    assert_eq!(status(&document, "cycle-a"), Status::Passed);
}

#[test]
fn the_report_names_every_case_of_the_regeneration() {
    let (mut roadmap, mut topics) = bundle();
    let mut document = document();
    pass(&mut document, "cycle-a");
    topic(&mut topics, "cycle-a").exam.focus = "Другой фокус".to_owned();
    roadmap.topics.retain(|entry| entry.id != "offline-edge");
    topics.retain(|topic| topic.id != "offline-edge");
    let mut fresh = topics[0].clone();
    fresh.id = "fresh-topic".to_owned();
    let mut entry = roadmap.topics[0].clone();
    entry.id = "fresh-topic".to_owned();
    roadmap.topics.push(entry);
    topics.push(fresh);

    let report = merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_eq!(report.stale.len(), 1);
    assert_eq!(report.added, ["fresh-topic"]);
    assert_eq!(report.orphaned, ["offline-edge"]);
    assert!(report.kept.contains(&"cycle-b".to_owned()));
    assert!(!report.kept.contains(&"offline-edge".to_owned()));
}

#[test]
fn the_merged_document_is_still_a_progress_file() {
    let (mut roadmap, mut topics) = bundle();
    let mut document = document();
    topic(&mut topics, "cycle-a")
        .questions
        .push(bundles::a_question());
    let mut fresh = topics[0].clone();
    fresh.id = "fresh-topic".to_owned();
    let mut entry = roadmap.topics[0].clone();
    entry.id = "fresh-topic".to_owned();
    roadmap.topics.push(entry);
    topics.push(fresh);

    merged(&mut document, &roadmap, &bundle().1, &topics);

    assert_eq!(&parse(document.text()).unwrap(), document.progress());
}

#[test]
fn merging_twice_changes_nothing_the_second_time() {
    let (roadmap, mut topics) = bundle();
    let mut document = document();
    pass(&mut document, "cycle-a");
    topic(&mut topics, "cycle-a").exam.focus = "Другой фокус".to_owned();

    merged(&mut document, &roadmap, &bundle().1, &topics);
    let once = document.text().to_owned();
    let report = merged(&mut document, &roadmap, &topics, &topics);

    assert_eq!(document.text(), once);
    assert_eq!(report.stale, []);
}
