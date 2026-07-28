use std::path::{Path, PathBuf};

use tolearn_core::bundle::validate;
use tolearn_core::merge::{self, Report};
use tolearn_core::progress::{Document, save};
use tolearn_core::registry::{Program, Registry};
use tolearn_core::scan::Scan;
use tolearn_core::topic::Topic;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{ImportIn, ImportOut, Merged, StaleTopic, Violation};
use crate::ipc::unpack::{self, Taken};
use crate::ipc::{history, open, settings};

pub fn run(context: &Context, input: &ImportIn) -> Result<ImportOut, IpcError> {
    let taken = unpack::taken(context, &input.path)?;
    let outcome = merged_in(context, input, &taken);
    if !matches!(&outcome, Ok(out) if out.ok) {
        unpack::discard(&taken);
    }
    outcome
}

fn merged_in(context: &Context, input: &ImportIn, taken: &Taken) -> Result<ImportOut, IpcError> {
    let scan = match open::read(&taken.root.display().to_string()) {
        Ok(scan) => scan,
        Err(error) => return Ok(refused(vec![violation(&error)])),
    };
    let violations = validate(&scan.roadmap, &scan.topics);
    if !violations.is_empty() {
        return Ok(refused(
            violations
                .iter()
                .map(|violation| Violation {
                    code: violation.code().to_owned(),
                    message: violation.to_string(),
                })
                .collect(),
        ));
    }

    let file = context.registry();
    let mut registry = Registry::read(&file)?;
    let previous = registry
        .programs()
        .iter()
        .find(|program| program.id == scan.roadmap.id)
        .map(|program| program.path.clone());

    if let Some(known) = previous.as_deref().filter(|path| path.is_dir()) {
        history::keep(
            &context.history(&scan.roadmap.id),
            known,
            &input.today,
            &settings::stored(context)?,
        )?;
    }

    let (mut document, before) = source(&scan, previous.as_deref())?;
    let report = merge::merge(&mut document, &scan.roadmap, &before, &scan.topics)?;
    save(&open::progress_file(&scan), &document)
        .map_err(|error| IpcError::unwritable(&scan.root, &error.to_string()))?;

    let home = unpack::settle(taken, &context.unpacked().join(&scan.roadmap.id))?;
    registry.add(Program {
        id: scan.roadmap.id.clone(),
        title: scan.roadmap.title.clone(),
        path: home,
        opened_at: Some(input.today.clone()),
    });
    registry.save(&file)?;

    Ok(ImportOut {
        ok: true,
        id: Some(scan.roadmap.id.clone()),
        title: Some(scan.roadmap.title.clone()),
        violations: Vec::new(),
        report: Some(merged(&report)),
    })
}

fn source(scan: &Scan, previous: Option<&Path>) -> Result<(Document, Vec<Topic>), IpcError> {
    let known: Option<PathBuf> = previous.filter(|path| path.is_dir()).map(Path::to_path_buf);
    let Some(known) = known else {
        let document = open::open(&scan.root.display().to_string())?.document;
        return Ok((document, scan.topics.clone()));
    };
    let opened = open::open(&known.display().to_string())?;
    Ok((opened.document, opened.scan.topics))
}

fn refused(violations: Vec<Violation>) -> ImportOut {
    ImportOut {
        ok: false,
        id: None,
        title: None,
        violations,
        report: None,
    }
}

fn violation(error: &IpcError) -> Violation {
    Violation {
        code: error.code.clone(),
        message: error.message.clone(),
    }
}

fn merged(report: &Report) -> Merged {
    Merged {
        kept: report.kept.clone(),
        added: report.added.clone(),
        orphaned: report.orphaned.clone(),
        stale: report
            .stale
            .iter()
            .map(|stale| StaleTopic {
                id: stale.id.clone(),
                changed: stale
                    .changed
                    .iter()
                    .map(|part| part.label().to_owned())
                    .collect(),
            })
            .collect(),
    }
}
