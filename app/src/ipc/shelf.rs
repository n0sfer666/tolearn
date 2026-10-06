use tolearn_core::Hours;
use tolearn_core::program::{Branch, Tree};
use tolearn_core::state::{State, Status};

use super::error::IpcError;
use super::reading::{RowView, StageRowView, SummaryView};
use super::types::Span;

pub fn branch<'a>(tree: &'a Tree, node: &str) -> Result<Branch<'a>, IpcError> {
    let uuid = Some(node)
        .filter(|node| !node.is_empty())
        .unwrap_or(tree.program.uuid.as_str());
    tree.branch(uuid).ok_or_else(|| {
        IpcError::new(
            "node.absent",
            format!("подпрограммы `{uuid}` в программе нет или она ещё не сгенерирована"),
        )
    })
}

pub fn span(hours: Hours) -> Span {
    Span {
        min: hours.min,
        max: hours.max,
    }
}

pub fn stages(tree: &Tree, state: &State) -> Vec<StageRowView> {
    tree.program
        .map
        .stages
        .iter()
        .map(|row| {
            let status = state.status(&tree.program.uuid, &row.id);
            StageRowView {
                id: row.id.clone(),
                title: row.title.clone(),
                hours: span(row.hours),
                ready: tree.stages.contains_key(&row.id),
                status: status.label().to_owned(),
                pass: if let Status::Passed(by) = status {
                    Some(by.label().to_owned())
                } else {
                    None
                },
            }
        })
        .collect()
}

pub fn summary(tree: &Tree, state: &State) -> SummaryView {
    let summary = state.summary(tree.every_stage());
    SummaryView {
        passed: summary.passed,
        total: summary.total,
        skipped: summary.skipped,
    }
}

pub fn children(tree: &Tree, state: &State) -> Vec<RowView> {
    tree.program
        .map
        .children
        .iter()
        .map(|row| {
            let child = tree.children.get(&row.uuid);
            RowView {
                id: row.uuid.clone(),
                title: row.title.clone(),
                hours: span(row.hours),
                ready: child.is_some(),
                summary: child.map(|child| summary(child, state)),
            }
        })
        .collect()
}
