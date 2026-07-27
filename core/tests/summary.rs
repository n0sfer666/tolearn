#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "summary gate: a panic here is the report"
)]

mod support;

use tolearn_core::status::effective;
use tolearn_core::summary::{Summary, summarize};
use tolearn_core::{Date, Hours};

use support::bundles;

fn day(text: &str) -> Date {
    Date::parse(text).unwrap_or_else(|| panic!("`{text}` is no date"))
}

fn whole(passed: &[&str]) -> Summary {
    sum(bundles::corpus_whole(), passed)
}

fn partial(passed: &[&str]) -> Summary {
    sum(bundles::corpus(), passed)
}

fn sum(
    bundle: (
        tolearn_core::roadmap::Roadmap,
        Vec<tolearn_core::topic::Topic>,
    ),
    passed: &[&str],
) -> Summary {
    let (map, topics) = bundle;
    let pairs: Vec<(&str, &str)> = passed.iter().map(|id| (*id, "passed")).collect();
    let state = bundles::recorded(&pairs);
    let statuses = effective(&map, &topics, &state, day("2026-07-27"));
    summarize(&map, &topics, &statuses)
}

fn stage(summary: &Summary, n: u32) -> &tolearn_core::summary::Stage {
    summary
        .stages
        .iter()
        .find(|stage| stage.n == n)
        .unwrap_or_else(|| panic!("stage {n} is not in the summary"))
}

#[test]
fn the_denominator_leaves_optional_topics_out() {
    let summary = whole(&[]);

    assert_eq!(summary.program.total, 4);
    assert_eq!(summary.program.done, 0);
}

#[test]
fn a_topic_the_generator_has_not_written_is_out_of_the_denominator() {
    let summary = partial(&[]);

    assert_eq!(summary.program.total, 3);
    assert_eq!(stage(&summary, 2).tally.total, 0);
}

#[test]
fn a_passed_topic_counts_as_done() {
    let summary = whole(&["cycle-a", "cycle-b"]);

    assert_eq!(summary.program.done, 2);
    assert_eq!(summary.program.total, 4);
}

#[test]
fn a_stale_pass_counts_as_done_and_is_named_separately() {
    let summary = whole(&["stale-knowledge"]);

    assert_eq!(summary.program.done, 1);
    assert_eq!(summary.program.stale, 1);
}

#[test]
fn a_fresh_pass_is_not_counted_as_stale() {
    let summary = whole(&["cycle-a", "cycle-b"]);

    assert_eq!(summary.program.stale, 0);
}

#[test]
fn hours_are_a_range_of_what_is_done_and_of_the_whole_program() {
    let summary = whole(&["cycle-a", "cycle-b"]);

    assert_eq!(summary.program.hours_done, Hours { min: 2, max: 4 });
    assert_eq!(summary.program.hours_total, Hours { min: 5, max: 9 });
}

#[test]
fn the_hours_of_an_optional_topic_are_out_of_the_total() {
    let summary = whole(&[]);

    assert_eq!(summary.program.hours_total, Hours { min: 5, max: 9 });
}

#[test]
fn the_share_is_the_done_out_of_the_denominator() {
    let summary = whole(&["cycle-a", "cycle-b"]);

    assert!((summary.program.share() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn a_program_with_nothing_to_count_has_a_share_of_zero() {
    let summary = partial(&[]);

    assert!((stage(&summary, 2).tally.share() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn every_stage_of_the_roadmap_has_a_row() {
    let summary = whole(&[]);

    let rows: Vec<u32> = summary.stages.iter().map(|stage| stage.n).collect();
    assert_eq!(rows, [1, 2]);
    assert_eq!(stage(&summary, 1).title, "Циклы и сроки");
}

#[test]
fn a_stage_counts_only_its_own_topics() {
    let summary = whole(&["cycle-a"]);

    assert_eq!(stage(&summary, 1).tally.total, 3);
    assert_eq!(stage(&summary, 1).tally.done, 1);
    assert_eq!(
        stage(&summary, 1).tally.hours_total,
        Hours { min: 4, max: 7 }
    );
    assert_eq!(stage(&summary, 2).tally.total, 1);
    assert_eq!(stage(&summary, 2).tally.done, 0);
}

#[test]
fn an_optional_topic_passed_out_by_calibration_still_counts_nowhere() {
    let (map, topics) = bundles::corpus_whole();
    let state = bundles::recorded(&[]);
    let statuses = effective(&map, &topics, &state, day("2026-07-27"));

    let summary = summarize(&map, &topics, &statuses);

    assert_eq!(summary.program.done, 0);
    assert_eq!(
        statuses.get("offline-edge"),
        Some(tolearn_core::progress::Status::PassedOut)
    );
}
