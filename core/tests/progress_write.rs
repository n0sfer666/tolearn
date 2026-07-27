#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::progress::{
    Answer, Attempt, Document, DocumentError, Format, NextAction, Outcome, Source, Verdict, parse,
};

use support::read;

const HISTORY: &str = "fixtures/valid/progress/attempts-history.yaml";

fn attempt() -> Attempt {
    Attempt {
        at: "2026-07-27T10:00:00Z".to_owned(),
        source: Source::Exam,
        verdict: Verdict::Pass,
        model: Some("claude-opus-5".to_owned()),
        hinted: false,
        practice_accepted: true,
        failed_checks: vec!["c1".to_owned()],
        per_question: vec![Answer {
            id: "q1".to_owned(),
            outcome: Outcome::Ok,
            quote: Some("Кэш контекста растёт вместе с длиной запроса.\n".to_owned()),
            missed: Vec::new(),
            signal_extension: true,
        }],
        gaps: Vec::new(),
        calibration: Some("Оценка совпала с результатом.\n".to_owned()),
        notes: vec!["Без подсказок.\n".to_owned()],
        next_action: Some(NextAction::Proceed),
        retry_after_days: None,
        raw: "{\"topic_id\": \"openai-compatible-api\", \"verdict\": \"pass\"}\n".to_owned(),
    }
}

fn document() -> Document {
    Document::read(&read(HISTORY), Format::Yaml).unwrap()
}

#[test]
fn a_document_left_alone_renders_into_the_same_progress() {
    let source = read(HISTORY);
    let document = Document::read(&source, Format::Yaml).unwrap();

    assert_eq!(parse(document.text()).unwrap(), parse(&source).unwrap());
}

#[test]
fn an_appended_attempt_reads_back_field_for_field() {
    let mut document = document();
    document
        .push_attempt("openai-compatible-api", &attempt())
        .unwrap();

    let written = parse(document.text()).unwrap();
    let state = written.state("openai-compatible-api").unwrap();
    assert_eq!(state.attempts, [attempt()]);
}

#[test]
fn an_appended_attempt_goes_after_the_attempts_that_were_there() {
    let mut document = document();
    document.push_attempt("local-runtime", &attempt()).unwrap();

    let written = parse(document.text()).unwrap();
    let attempts = &written.state("local-runtime").unwrap().attempts;
    assert_eq!(attempts.len(), 3);
    assert_eq!(attempts[0].at, "2026-07-20T11:05:00Z");
    assert_eq!(attempts[2], attempt());
}

#[test]
fn the_raw_answer_survives_the_rewrite_word_for_word() {
    let before = parse(&read(HISTORY)).unwrap();
    let mut document = document();
    document
        .push_attempt("openai-compatible-api", &attempt())
        .unwrap();
    let after = parse(document.text()).unwrap();

    assert_eq!(
        after.state("local-runtime").unwrap().attempts[0].raw,
        before.state("local-runtime").unwrap().attempts[0].raw
    );
}

#[test]
fn the_order_of_the_topics_holds_through_a_rewrite() {
    let source = read(HISTORY);
    let mut document = document();
    document
        .push_attempt("structured-output", &attempt())
        .unwrap();

    assert_eq!(written_order(document.text()), written_order(&source));
    assert_eq!(
        written_order(document.text()),
        [
            "local-runtime",
            "openai-compatible-api",
            "tokens-context-cost",
            "structured-output"
        ]
    );
}

fn written_order(text: &str) -> Vec<String> {
    text.lines()
        .skip_while(|line| *line != "topics:")
        .filter_map(|line| {
            let name = line.strip_prefix("  ")?.strip_suffix(':')?;
            (!name.starts_with(' ') && !name.starts_with('-')).then(|| name.to_owned())
        })
        .collect()
}

#[test]
fn a_key_this_version_does_not_know_is_left_where_it_was() {
    let source = "\
schema: learning-roadmap/progress/v1
roadmap_id: minimal-program
minutes_spent: 41
topics:
  minimal-topic:
    status: todo
    attempts: []
    passed_at: null
    next_review_at: null
    gaps: []
    mood: calm
";
    let mut document = Document::read(source, Format::Yaml).unwrap();
    document.push_attempt("minimal-topic", &attempt()).unwrap();

    let written = document.text();
    assert!(written.contains("minutes_spent: 41"), "{written}");
    assert!(written.contains("mood: calm"), "{written}");
}

#[test]
fn an_attempt_for_a_topic_the_file_does_not_carry_is_refused() {
    let mut document = document();
    let error = document
        .push_attempt("no-such-topic", &attempt())
        .unwrap_err();

    assert_eq!(
        error,
        DocumentError::UnknownTopic("no-such-topic".to_owned())
    );
    assert!(format!("{error}").contains("no-such-topic"));
}

#[test]
fn a_json_document_is_written_back_as_json() {
    let source = read("examples/llm-agents-base/progress.yaml");
    let yaml = Document::read(&source, Format::Yaml).unwrap();
    let mut json = Document::read(&source, Format::Json).unwrap();
    json.push_attempt("local-runtime", &attempt()).unwrap();

    assert!(json.text().starts_with('{'), "{}", json.text());
    assert!(!yaml.text().starts_with('{'), "{}", yaml.text());
    assert_eq!(
        parse(json.text())
            .unwrap()
            .state("local-runtime")
            .unwrap()
            .attempts,
        [attempt()]
    );
}

#[test]
fn a_document_rendered_in_either_format_holds_the_same_progress() {
    let source = read(HISTORY);
    let yaml = Document::read(&source, Format::Yaml).unwrap();
    let json = Document::read(&source, Format::Json).unwrap();

    assert_eq!(parse(json.text()).unwrap(), parse(yaml.text()).unwrap());
}

#[test]
fn starting_a_topic_that_is_already_in_the_file_leaves_it_alone() {
    let mut document = Document::read(&read(HISTORY), Format::Yaml).unwrap();
    let before = document.text().to_owned();

    document.start("local-runtime").unwrap();

    assert_eq!(document.text(), before);
}
