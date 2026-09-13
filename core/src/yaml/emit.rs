use std::borrow::Cow;
use std::fmt;

use saphyr::{Scalar, Yaml, YamlEmitter};

pub(crate) fn text(value: &str) -> Yaml<'_> {
    Yaml::Value(Scalar::String(Cow::Borrowed(value)))
}

pub(crate) fn flag(value: bool) -> Yaml<'static> {
    Yaml::Value(Scalar::Boolean(value))
}

pub(crate) fn number(value: u32) -> Yaml<'static> {
    Yaml::Value(Scalar::Integer(i64::from(value)))
}

pub(crate) fn list<'a>(items: impl IntoIterator<Item = Yaml<'a>>) -> Yaml<'a> {
    Yaml::Sequence(items.into_iter().collect())
}

pub(crate) fn map<'a>(entries: impl IntoIterator<Item = (&'a str, Yaml<'a>)>) -> Yaml<'a> {
    Yaml::Mapping(
        entries
            .into_iter()
            .map(|(key, value)| (text(key), value))
            .collect(),
    )
}

pub fn dump(node: &Yaml<'_>) -> Result<String, fmt::Error> {
    let mut out = String::new();
    let mut emitter = YamlEmitter::new(&mut out);
    emitter.multiline_strings(blockable(node));
    emitter.dump(node).map_err(|_| fmt::Error)?;
    let body = out.strip_prefix("---\n").unwrap_or(&out);
    Ok(format!("{body}\n"))
}

fn blockable(node: &Yaml<'_>) -> bool {
    match node {
        Yaml::Value(Scalar::String(value)) => !value.contains('\n') || literal(value),
        Yaml::Sequence(items) => items.iter().all(blockable),
        Yaml::Mapping(entries) => entries.values().all(blockable),
        _ => true,
    }
}

fn literal(value: &str) -> bool {
    !value.contains('\r')
        && !value.ends_with("\n\n")
        && value
            .lines()
            .next()
            .is_some_and(|first| !first.is_empty() && !first.starts_with([' ', '\t']))
}
