use tolearn_core::program::{Program, StageRow, Tree};

use super::after::After;
use super::error::NextError;

#[derive(Debug, Clone)]
pub(super) struct Ahead {
    pub(super) tree: Tree,
    pub(super) leaf: Program,
    pub(super) prefix: String,
    pub(super) index: usize,
    pub(super) next: StageRow,
}

pub(super) fn ahead(tree: Tree, after: &After<'_>) -> Result<Ahead, NextError> {
    let branch = tree
        .branch(after.node)
        .ok_or_else(|| NextError::Node(after.node.to_owned()))?;
    let node = branch.tree;
    let stages = &node.program.map.stages;
    let index = stages
        .iter()
        .position(|row| row.id == after.stage)
        .ok_or_else(|| NextError::Stage(after.stage.to_owned()))?;
    if !node.stages.contains_key(after.stage) {
        return Err(NextError::Ungenerated(after.stage.to_owned()));
    }
    let next = stages
        .get(index + 1)
        .ok_or_else(|| NextError::End(after.stage.to_owned()))?;
    if node.stages.contains_key(&next.id) {
        return Err(NextError::Taken(next.id.clone()));
    }
    let (leaf, prefix, next) = (node.program.clone(), branch.prefix.clone(), next.clone());
    Ok(Ahead {
        tree,
        leaf,
        prefix,
        index,
        next,
    })
}
