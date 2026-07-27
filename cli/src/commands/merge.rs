use std::path::Path;

use serde_json::{Value, json};
use tolearn_core::merge::{Part, Report, merge};
use tolearn_core::progress::save;

use crate::bundle;
use crate::error::CliError;
use crate::out::Output;

pub fn run(root: &Path, was: &Path) -> Result<Output, CliError> {
    let mut opened = bundle::open(root)?;
    let before = bundle::read(was)?;
    let report = merge(
        &mut opened.document,
        &opened.scan.roadmap,
        &before.topics,
        &opened.scan.topics,
    )
    .map_err(|error| CliError::Write(error.to_string()))?;
    save(&opened.progress_file, &opened.document)
        .map_err(|error| CliError::Write(error.to_string()))?;
    Ok(Output::new(text(&report), body(&report)))
}

fn text(report: &Report) -> String {
    let mut out = format!(
        "оставлено: {}, добавлено: {}, протухло: {}, осиротело: {}\n",
        report.kept.len(),
        report.added.len(),
        report.stale.len(),
        report.orphaned.len()
    );
    for stale in &report.stale {
        let changed: Vec<&str> = stale.changed.iter().copied().map(Part::label).collect();
        out.push_str(&format!("  {} — {}\n", stale.id, changed.join(", ")));
    }
    out
}

fn body(report: &Report) -> Value {
    json!({
        "kept": report.kept,
        "added": report.added,
        "orphaned": report.orphaned,
        "stale": report.stale.iter().map(|stale| json!({
            "id": stale.id,
            "changed": stale.changed.iter().copied().map(Part::label).collect::<Vec<&str>>(),
        })).collect::<Vec<Value>>(),
    })
}
