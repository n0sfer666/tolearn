use std::collections::BTreeMap;

use super::types::{
    Calibration, CalibrationMethod, Defaults, Priority, RevalidateAfterDays, Roadmap, Stage,
    TopicEntry,
};
use crate::hours::hours;
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
        generated_at: node.field("generated_at")?.date()?,
        generated_by: node.field("generated_by")?.text()?,
        locale: node.field("locale")?.text()?,
        weekly_hours: node.field("weekly_hours")?.number(1)?,
        env_constraints: node.field("env_constraints")?.texts()?,
        version_pins: pins(&node.field("version_pins")?)?,
        calibration: calibration(&node.field("calibration")?)?,
        defaults: defaults(&node.field("defaults")?)?,
        stages: node.field("stages")?.list(stage)?,
        topics: node.field("topics")?.list(topic)?,
    })
}

fn calibration(node: &Reader<'_>) -> Result<Calibration, ParseError> {
    Ok(Calibration {
        method: node.field("method")?.choice(
            "calibration method",
            &[
                ("diagnostic-probe", CalibrationMethod::DiagnosticProbe),
                ("self-report", CalibrationMethod::SelfReport),
                ("none", CalibrationMethod::None),
            ],
        )?,
        probed: node.field("probed")?.number(0)?,
        passed_out: node.field("passed_out")?.texts()?,
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
        priority: node.field("priority")?.choice(
            "priority",
            &[
                ("core", Priority::Core),
                ("recommended", Priority::Recommended),
                ("optional", Priority::Optional),
            ],
        )?,
    })
}

fn pins(node: &Reader<'_>) -> Result<BTreeMap<String, String>, ParseError> {
    node.entries()?
        .into_iter()
        .map(|(name, value)| Ok((name, value.text()?)))
        .collect()
}
