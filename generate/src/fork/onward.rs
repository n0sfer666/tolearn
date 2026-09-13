use tolearn_core::program::{ChildRow, Program, StageRow, Tree};

use crate::plan::{Part, Request};

use super::after::After;
use super::error::NextError;
use super::offered::Fork;
use super::variant::Variant;

const CHILDREN: &str = "children";

#[derive(Debug, Clone)]
pub(super) struct Onward {
    pub(super) tree: Tree,
    pub(super) parent: Program,
    pub(super) prefix: String,
    pub(super) row: ChildRow,
    pub(super) depth: usize,
}

impl Onward {
    pub(super) fn part(&self) -> Part {
        Part {
            title: self.row.title.clone(),
            goal: self
                .row
                .goal
                .clone()
                .unwrap_or_else(|| self.parent.goal.clone()),
            hours: self.row.hours,
        }
    }

    pub(super) fn request(&self) -> Request {
        let root = &self.tree.program;
        Request {
            request: root.generation.request.clone(),
            level: root.level.clone(),
            locale: root.generation.locale.clone(),
        }
    }

    pub(super) fn fork(&self) -> Fork {
        let Part { title, goal, hours } = self.part();
        let row = StageRow {
            id: self.row.uuid.clone(),
            title,
            hours,
        };
        Fork {
            variants: vec![Variant {
                row,
                why: goal,
                recommended: true,
            }],
        }
    }
}

struct Next<'a> {
    parent: &'a Tree,
    row: &'a ChildRow,
    prefix: String,
    depth: usize,
}

pub(super) fn onward(tree: Tree, after: &After<'_>) -> Result<Onward, NextError> {
    let Next {
        parent,
        row,
        prefix,
        depth,
    } = next(&tree, after.node).ok_or_else(|| NextError::End(after.stage.to_owned()))?;
    if parent.children.contains_key(&row.uuid) {
        return Err(NextError::Begun(row.title.clone()));
    }
    let (parent, row) = (parent.program.clone(), row.clone());
    Ok(Onward {
        tree,
        parent,
        prefix,
        row,
        depth,
    })
}

fn next<'a>(tree: &'a Tree, node: &str) -> Option<Next<'a>> {
    let branch = tree.branch(node)?;
    let mut current = node;
    for (level, parent) in branch.trail.iter().copied().enumerate().rev() {
        let rows = &parent.program.map.children;
        let position = rows.iter().position(|row| row.uuid == current)?;
        if let Some(row) = rows.get(position + 1) {
            let above: String = branch
                .trail
                .iter()
                .skip(1)
                .take(level)
                .map(|tree| format!("{CHILDREN}/{}/", tree.program.uuid))
                .collect();
            return Some(Next {
                parent,
                row,
                prefix: format!("{above}{CHILDREN}/{}", row.uuid),
                depth: level + 2,
            });
        }
        current = &parent.program.uuid;
    }
    None
}
