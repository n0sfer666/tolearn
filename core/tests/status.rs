#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "status gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::progress::{Progress, Status, Verdict, parse as progress};
use tolearn_core::roadmap::Roadmap;
use tolearn_core::status::{Statuses, effective, from_verdict, next_review_at};
use tolearn_core::topic::Topic;

use support::bundles;
use support::read;

const CORPUS_PROGRESS: &str = "fixtures/valid/progress/corpus-program.yaml";

fn day(text: &str) -> Date {
    Date::parse(text).unwrap_or_else(|| panic!("`{text}` is no date"))
}

fn corpus() -> (Roadmap, Vec<Topic>, Progress) {
    let (map, topics) = bundles::corpus_whole();
    let state = progress(&read(CORPUS_PROGRESS)).unwrap();
    (map, topics, state)
}

fn recorded(pairs: &[(&str, &str)]) -> Progress {
    let head = "schema: learning-roadmap/progress/v1\nroadmap_id: corpus-program\ntopics:";
    let mut source = format!("{head}{}\n", if pairs.is_empty() { " {}" } else { "" });
    for (id, status) in pairs {
        source.push_str(&format!(
            "  {id}:\n    status: {status}\n    attempts: []\n    passed_at: null\n    next_review_at: null\n    gaps: []\n"
        ));
    }
    progress(&source).unwrap()
}

fn of(statuses: &Statuses, id: &str) -> Status {
    statuses
        .get(id)
        .unwrap_or_else(|| panic!("`{id}` has no status"))
}

fn document<'a>(topics: &'a [Topic], id: &str) -> &'a Topic {
    &topics[bundles::document(topics, id)]
}

#[test]
fn a_topic_whose_dependency_is_unfinished_is_blocked() {
    let (map, topics, state) = corpus();

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "cycle-a"), Status::Blocked);
    assert_eq!(of(&statuses, "cycle-b"), Status::Blocked);
}

#[test]
fn a_dependency_that_is_done_opens_the_topic() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("cycle-a", "passed"), ("cycle-b", "todo")]);

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "cycle-b"), Status::Todo);
}

#[test]
fn a_dependency_that_is_only_started_still_blocks() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("cycle-a", "in_progress"), ("cycle-b", "todo")]);

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "cycle-b"), Status::Blocked);
}

#[test]
fn a_dependency_whose_knowledge_went_stale_still_counts_as_done() {
    let (map, topics, state) = corpus();

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "stale-knowledge"), Status::StalePassed);
    assert_eq!(of(&statuses, "offline-edge"), Status::PassedOut);
}

#[test]
fn a_topic_that_is_passed_is_not_blocked_by_what_it_waited_for() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("cycle-a", "passed"), ("cycle-b", "passed")]);

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "cycle-a"), Status::Passed);
}

#[test]
fn a_dependency_whose_knowledge_is_stale_opens_the_topic_all_the_same() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("cycle-a", "passed"), ("cycle-b", "todo")]);

    let statuses = effective(&map, &topics, &state, day("2028-07-27"));

    assert_eq!(of(&statuses, "cycle-a"), Status::StalePassed);
    assert_eq!(of(&statuses, "cycle-b"), Status::Todo);
}

#[test]
fn a_blocked_mark_left_in_the_file_goes_away_with_the_dependency() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("cycle-a", "passed"), ("cycle-b", "blocked")]);

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "cycle-b"), Status::Todo);
}

#[test]
fn a_topic_the_file_says_nothing_about_starts_at_todo() {
    let (map, topics, _) = corpus();
    let state = recorded(&[]);

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "stale-knowledge"), Status::Todo);
}

#[test]
fn every_topic_of_the_roadmap_gets_a_status_in_the_order_of_the_roadmap() {
    let (map, topics, state) = corpus();

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    let listed: Vec<&str> = statuses.iter().map(|(id, _)| id.as_str()).collect();
    let expected: Vec<&str> = map.topics.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(listed, expected);
}

#[test]
fn knowledge_goes_stale_on_the_day_its_term_runs_out() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("stale-knowledge", "passed")]);

    let fresh = effective(&map, &topics, &state, day("2026-05-30"));
    let stale = effective(&map, &topics, &state, day("2026-05-31"));

    assert_eq!(of(&fresh, "stale-knowledge"), Status::Passed);
    assert_eq!(of(&stale, "stale-knowledge"), Status::StalePassed);
}

#[test]
fn the_term_of_the_topic_beats_the_defaults_of_the_roadmap() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("stale-knowledge", "passed")]);
    let topic = document(&topics, "stale-knowledge");

    assert_eq!(topic.revalidate_after_days, 30);
    assert_eq!(map.defaults.revalidate_after_days.evolving, 365);

    let statuses = effective(&map, &topics, &state, day("2026-05-31"));

    assert_eq!(
        of(&statuses, "stale-knowledge"),
        Status::StalePassed,
        "365 days of the header would still call it fresh"
    );
}

#[test]
fn a_topic_that_was_never_passed_does_not_go_stale() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("stale-knowledge", "todo")]);

    let statuses = effective(&map, &topics, &state, day("2030-01-01"));

    assert_eq!(of(&statuses, "stale-knowledge"), Status::Todo);
}

#[test]
fn a_topic_passed_out_by_calibration_is_passed_out_whatever_the_file_says() {
    let (map, topics, _) = corpus();
    let state = recorded(&[("offline-edge", "todo")]);

    assert_eq!(map.calibration.passed_out, ["offline-edge"]);

    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    assert_eq!(of(&statuses, "offline-edge"), Status::PassedOut);
}

#[test]
fn a_topic_passed_out_by_calibration_goes_stale_like_any_other() {
    let (map, topics, _) = corpus();
    let state = recorded(&[]);

    let statuses = effective(&map, &topics, &state, day("2026-10-18"));

    assert_eq!(of(&statuses, "offline-edge"), Status::StalePassed);
}

#[test]
fn the_next_review_of_a_topic_kept_by_schedule_counts_from_the_pass() {
    let (_, topics, _) = corpus();
    let topic = document(&topics, "stale-knowledge");

    let date = next_review_at(topic, Some(day("2026-06-01")));

    assert_eq!(date.map(|date| date.to_string()), Some("2026-07-01".into()));
}

#[test]
fn a_topic_kept_by_schedule_has_no_review_until_it_is_passed() {
    let (_, topics, _) = corpus();
    let topic = document(&topics, "stale-knowledge");

    assert_eq!(next_review_at(topic, None), None);
}

#[test]
fn the_next_review_of_a_topic_kept_by_use_counts_from_the_day_it_was_verified() {
    let (_, topics, _) = corpus();
    let topic = document(&topics, "offline-edge");

    let date = next_review_at(topic, None);

    assert_eq!(date.map(|date| date.to_string()), Some("2026-10-18".into()));
}

#[test]
fn a_topic_that_does_not_come_back_has_no_next_review() {
    let (_, topics, _) = corpus();
    let topic = document(&topics, "question-shapes");

    assert_eq!(next_review_at(topic, Some(day("2026-07-27"))), None);
}

#[test]
fn the_verdict_table_is_covered_whole() {
    fn expected(verdict: Verdict) -> Status {
        match verdict {
            Verdict::Pass => Status::Passed,
            Verdict::Partial => Status::InProgress,
            Verdict::Fail => Status::Failed,
            Verdict::Blocked => Status::InProgress,
        }
    }

    for verdict in [
        Verdict::Pass,
        Verdict::Partial,
        Verdict::Fail,
        Verdict::Blocked,
    ] {
        assert_eq!(from_verdict(verdict), expected(verdict), "{verdict:?}");
    }
}

#[test]
fn a_day_added_to_a_date_crosses_months_and_years() {
    assert_eq!(day("2026-05-01").plus_days(30).to_string(), "2026-05-31");
    assert_eq!(day("2026-12-20").plus_days(30).to_string(), "2027-01-19");
    assert_eq!(day("2024-02-28").plus_days(1).to_string(), "2024-02-29");
    assert_eq!(day("2026-02-28").plus_days(1).to_string(), "2026-03-01");
    assert_eq!(day("2026-01-31").plus_days(365).to_string(), "2027-01-31");
}

#[test]
fn dates_are_ordered_by_the_calendar() {
    assert!(day("2026-05-31") > day("2026-05-30"));
    assert!(day("2026-01-01") > day("2025-12-31"));
    assert_eq!(Date::parse("2026-13-01"), None);
}
