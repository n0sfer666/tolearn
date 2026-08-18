#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::progress::{Outcome, TopicState};
use tolearn_core::protocol::apply;
use tolearn_core::review::review;
use tolearn_core::topic::Topic;
use tolearn_core::verdict::{Verdict, parse};

use support::bundles;

const AT: &str = "2026-07-27T18:40:00+03:00";

fn topic() -> Topic {
    let (_, topics) = bundles::corpus_whole();
    topics[bundles::document(&topics, "stale-knowledge")].clone()
}

fn graded(subject: &Topic, result: &str, answers: &str, gaps: &str) -> Verdict {
    let text = format!(
        "{{\"topic_id\": \"{}\", \"date\": \"2026-07-27\", \"verdict\": \"{result}\",
          \"per_question\": {answers}, \"gaps\": {gaps}}}",
        subject.id
    );
    parse(&text, subject).unwrap()
}

fn state(subject: &Topic, rounds: &[(&str, &str, &str)]) -> TopicState {
    let today = Date::parse("2026-07-27").unwrap();
    let mut state = None;
    for (result, answers, gaps) in rounds {
        state = Some(
            apply(
                &graded(subject, result, answers, gaps),
                subject,
                state.as_ref(),
                AT,
                today,
            )
            .state,
        );
    }
    state.unwrap()
}

fn ids(subject: &Topic) -> Vec<String> {
    subject
        .questions
        .iter()
        .map(|question| question.id.clone())
        .collect()
}

#[test]
fn пробел_висит_на_том_вопросе_который_его_упустил() {
    let subject = topic();
    let asked = ids(&subject);
    let answers = format!(
        "[{{\"id\": \"{}\", \"result\": \"miss\", \"missed\": [\"порядок обхода\"]}}]",
        asked[0]
    );
    let state = state(&subject, &[("fail", &answers, "[\"порядок обхода\"]")]);

    let seen = review(&subject, Some(&state));

    let first = seen
        .questions
        .iter()
        .find(|question| question.id == asked[0])
        .unwrap();
    assert_eq!(first.outcome, Some(Outcome::Miss));
    assert_eq!(first.missed, vec!["порядок обхода".to_owned()]);
    assert!(seen.loose.is_empty(), "{:?}", seen.loose);
}

#[test]
fn пробел_ничей_показан_отдельно_а_не_потерян() {
    let subject = topic();
    let asked = ids(&subject);
    let answers = format!("[{{\"id\": \"{}\", \"result\": \"ok\"}}]", asked[0]);
    let state = state(&subject, &[("partial", &answers, "[\"общая картина\"]")]);

    let seen = review(&subject, Some(&state));

    assert_eq!(seen.loose, vec!["общая картина".to_owned()]);
}

#[test]
fn неотвеченный_вопрос_остаётся_в_списке_без_исхода() {
    let subject = topic();
    let asked = ids(&subject);
    let answers = format!("[{{\"id\": \"{}\", \"result\": \"ok\"}}]", asked[0]);
    let state = state(&subject, &[("partial", &answers, "[]")]);

    let seen = review(&subject, Some(&state));

    assert_eq!(seen.questions.len(), subject.questions.len());
    assert!(
        seen.questions
            .iter()
            .skip(1)
            .all(|question| question.outcome.is_none())
    );
}

#[test]
fn последняя_попытка_подробно_а_прошлые_строкой() {
    let subject = topic();
    let asked = ids(&subject);
    let answers = format!("[{{\"id\": \"{}\", \"result\": \"miss\"}}]", asked[0]);
    let state = state(
        &subject,
        &[
            ("fail", &answers, "[]"),
            ("fail", &answers, "[]"),
            ("partial", &answers, "[\"свежий пробел\"]"),
        ],
    );

    let seen = review(&subject, Some(&state));

    assert_eq!(seen.loose, vec!["свежий пробел".to_owned()]);
    assert_eq!(seen.history.len(), 2);
    assert!(
        seen.history
            .iter()
            .all(|past| past.verdict.label() == "fail")
    );
}

#[test]
fn три_провала_подряд_дают_готовый_текст_запроса() {
    let subject = topic();
    let asked = ids(&subject);
    let answers = format!(
        "[{{\"id\": \"{}\", \"result\": \"miss\", \"missed\": [\"порядок обхода\"]}}]",
        asked[0]
    );
    let round = ("fail", answers.as_str(), "[\"порядок обхода\"]");
    let state = state(&subject, &[round, round, round]);

    let seen = review(&subject, Some(&state));

    assert!(seen.split_suggested);
    assert!(
        seen.split_request.contains(&subject.id),
        "{}",
        seen.split_request
    );
    assert!(seen.split_request.contains(&subject.title));
    assert!(
        seen.split_request.contains("порядок обхода"),
        "{}",
        seen.split_request
    );
    assert!(
        seen.split_request.contains(&asked[0]),
        "{}",
        seen.split_request
    );
}

#[test]
fn двух_провалов_мало_и_текста_запроса_нет() {
    let subject = topic();
    let asked = ids(&subject);
    let answers = format!("[{{\"id\": \"{}\", \"result\": \"miss\"}}]", asked[0]);
    let round = ("fail", answers.as_str(), "[]");
    let state = state(&subject, &[round, round]);

    let seen = review(&subject, Some(&state));

    assert!(!seen.split_suggested);
    assert!(seen.split_request.is_empty());
}

#[test]
fn тема_без_попыток_показывает_вопросы_и_ничего_не_предлагает() {
    let subject = topic();

    let seen = review(&subject, None);

    assert_eq!(seen.questions.len(), subject.questions.len());
    assert!(seen.history.is_empty());
    assert!(!seen.split_suggested);
}
