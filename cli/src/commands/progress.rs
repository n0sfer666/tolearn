use std::path::Path;

use serde_json::{Value, json};
use tolearn_core::Date;
use tolearn_core::status::effective;
use tolearn_core::summary::{Summary, Tally, summarize};

use crate::bundle;
use crate::error::CliError;
use crate::out::Output;
use crate::today;

pub fn run(root: &Path, today: Option<&str>) -> Result<Output, CliError> {
    let opened = bundle::open(root)?;
    let day = day(today)?;
    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    let summary = summarize(&opened.scan.roadmap, &opened.scan.topics, &statuses);
    Ok(Output::new(text(&summary), body(&summary)))
}

fn day(given: Option<&str>) -> Result<Date, CliError> {
    match given {
        Some(value) => Date::parse(value)
            .ok_or_else(|| CliError::Usage(format!("`--today {value}` — не дата"))),
        None => today::today().ok_or_else(|| CliError::Bundle("часы системы врут".to_owned())),
    }
}

fn text(summary: &Summary) -> String {
    let mut out = format!("Программа: {}\n", line(&summary.program));
    for stage in &summary.stages {
        out.push_str(&format!(
            "Этап {} «{}»: {}\n",
            stage.n,
            stage.title,
            line(&stage.tally)
        ));
    }
    out
}

fn line(tally: &Tally) -> String {
    let percent = (tally.share() * 100.0).round();
    let stale = if tally.stale > 0 {
        format!(", протухло {}", tally.stale)
    } else {
        String::new()
    };
    format!(
        "{}/{} ({percent:.0}%), часов {}–{} из {}–{}{stale}",
        tally.done,
        tally.total,
        tally.hours_done.min,
        tally.hours_done.max,
        tally.hours_total.min,
        tally.hours_total.max
    )
}

fn body(summary: &Summary) -> Value {
    json!({
        "program": tally(&summary.program),
        "stages": summary.stages.iter().map(|stage| json!({
            "n": stage.n,
            "title": stage.title,
            "tally": tally(&stage.tally),
        })).collect::<Vec<Value>>(),
    })
}

fn tally(tally: &Tally) -> Value {
    json!({
        "done": tally.done,
        "total": tally.total,
        "stale": tally.stale,
        "share": tally.share(),
        "hours_done": [tally.hours_done.min, tally.hours_done.max],
        "hours_total": [tally.hours_total.min, tally.hours_total.max],
    })
}
