use std::borrow::Cow;

use saphyr::{Mapping, Scalar, Yaml};

use super::types::{Answer, Attempt};

pub fn attempt(attempt: &Attempt) -> Yaml<'static> {
    let mut fields = vec![
        ("at", text(&attempt.at)),
        ("source", text(attempt.source.label())),
        ("verdict", text(attempt.verdict.label())),
        ("model", maybe_text(attempt.model.as_deref())),
        ("hinted", flag(attempt.hinted)),
        ("practice_accepted", flag(attempt.practice_accepted)),
        ("failed_checks", texts(&attempt.failed_checks)),
        (
            "per_question",
            Yaml::Sequence(attempt.per_question.iter().map(answer).collect()),
        ),
        ("gaps", texts(&attempt.gaps)),
        ("calibration", maybe_text(attempt.calibration.as_deref())),
        ("notes", texts(&attempt.notes)),
    ];
    if let Some(action) = attempt.next_action {
        fields.push(("next_action", text(action.label())));
    }
    if let Some(days) = attempt.retry_after_days {
        fields.push(("retry_after_days", number(days)));
    }
    fields.push(("raw", text(&attempt.raw)));
    mapping(fields)
}

fn answer(answer: &Answer) -> Yaml<'static> {
    mapping(vec![
        ("id", text(&answer.id)),
        ("result", text(answer.outcome.label())),
        ("quote", maybe_text(answer.quote.as_deref())),
        ("missed", texts(&answer.missed)),
        ("signal_extension", flag(answer.signal_extension)),
    ])
}

fn mapping(fields: Vec<(&str, Yaml<'static>)>) -> Yaml<'static> {
    let mut map = Mapping::new();
    for (key, value) in fields {
        map.insert(text(key), value);
    }
    Yaml::Mapping(map)
}

pub(super) fn text(value: &str) -> Yaml<'static> {
    Yaml::Value(Scalar::String(Cow::Owned(value.to_owned())))
}

pub(super) fn maybe_text(value: Option<&str>) -> Yaml<'static> {
    value.map_or_else(|| Yaml::Value(Scalar::Null), text)
}

fn texts(values: &[String]) -> Yaml<'static> {
    Yaml::Sequence(values.iter().map(|value| text(value)).collect())
}

fn flag(value: bool) -> Yaml<'static> {
    Yaml::Value(Scalar::Boolean(value))
}

fn number(value: u32) -> Yaml<'static> {
    Yaml::Value(Scalar::Integer(i64::from(value)))
}
