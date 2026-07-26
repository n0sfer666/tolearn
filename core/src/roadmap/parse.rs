use std::collections::BTreeMap;

use super::types::{
    Calibration, CalibrationMethod, Defaults, Hours, Priority, RevalidateAfterDays, Roadmap, Stage,
    TopicEntry,
};
use crate::yaml::{ParseError, Reader, read};

pub fn parse(source: &str) -> Result<Roadmap, ParseError> {
    read(source, roadmap)
}

fn roadmap(node: &Reader<'_>) -> Result<Roadmap, ParseError> {
    Ok(Roadmap {
        schema: node.field("schema")?.text()?,
        id: node.field("id")?.text()?,
        title: node.field("title")?.text()?,
        subject: node.field("subject")?.text()?,
        goal: node.field("goal")?.text()?,
        generated_at: node.field("generated_at")?.text()?,
        generated_by: node.field("generated_by")?.text()?,
        locale: node.field("locale")?.text()?,
        weekly_hours: node.field("weekly_hours")?.number(1)?,
        env_constraints: texts(&node.field("env_constraints")?)?,
        version_pins: pins(&node.field("version_pins")?)?,
        calibration: calibration(&node.field("calibration")?)?,
        defaults: defaults(&node.field("defaults")?)?,
        stages: node
            .field("stages")?
            .items()?
            .iter()
            .map(stage)
            .collect::<Result<_, _>>()?,
        topics: node
            .field("topics")?
            .items()?
            .iter()
            .map(topic)
            .collect::<Result<_, _>>()?,
    })
}

fn calibration(node: &Reader<'_>) -> Result<Calibration, ParseError> {
    Ok(Calibration {
        method: method(&node.field("method")?)?,
        probed: node.field("probed")?.number(0)?,
        passed_out: texts(&node.field("passed_out")?)?,
        interrupted: node.field("interrupted")?.flag()?,
    })
}

fn defaults(node: &Reader<'_>) -> Result<Defaults, ParseError> {
    let days = node.field("revalidate_after_days")?;
    Ok(Defaults {
        revalidate_after_days: RevalidateAfterDays {
            stable: days.field("stable")?.number(1)?,
            evolving: days.field("evolving")?.number(1)?,
            volatile: days.field("volatile")?.number(1)?,
        },
    })
}

fn stage(node: &Reader<'_>) -> Result<Stage, ParseError> {
    Ok(Stage {
        n: node.field("n")?.number(1)?,
        title: node.field("title")?.text()?,
        generated: node.field("generated")?.flag()?,
        checkpoint: node.field("checkpoint")?.text()?,
    })
}

fn topic(node: &Reader<'_>) -> Result<TopicEntry, ParseError> {
    Ok(TopicEntry {
        id: node.field("id")?.text()?,
        title: node.field("title")?.text()?,
        stage: node.field("stage")?.number(1)?,
        file: node.field("file")?.text()?,
        est_hours: hours(&node.field("est_hours")?)?,
        priority: priority(&node.field("priority")?)?,
    })
}

fn hours(node: &Reader<'_>) -> Result<Hours, ParseError> {
    let items = node.items()?;
    let [min, max] = items.as_slice() else {
        return Err(node.malformed("expected a list of exactly two numbers"));
    };
    Ok(Hours {
        min: min.number(1)?,
        max: max.number(1)?,
    })
}

fn method(node: &Reader<'_>) -> Result<CalibrationMethod, ParseError> {
    match node.text()?.as_str() {
        "diagnostic-probe" => Ok(CalibrationMethod::DiagnosticProbe),
        "self-report" => Ok(CalibrationMethod::SelfReport),
        "none" => Ok(CalibrationMethod::None),
        other => Err(node.unknown(format!("unknown calibration method `{other}`"))),
    }
}

fn priority(node: &Reader<'_>) -> Result<Priority, ParseError> {
    match node.text()?.as_str() {
        "core" => Ok(Priority::Core),
        "recommended" => Ok(Priority::Recommended),
        "optional" => Ok(Priority::Optional),
        other => Err(node.unknown(format!("unknown priority `{other}`"))),
    }
}

fn texts(node: &Reader<'_>) -> Result<Vec<String>, ParseError> {
    node.items()?.iter().map(Reader::text).collect()
}

fn pins(node: &Reader<'_>) -> Result<BTreeMap<String, String>, ParseError> {
    node.entries()?
        .into_iter()
        .map(|(name, value)| Ok((name, value.text()?)))
        .collect()
}
