use tolearn_core::program::{Program, StageRow, Tree};

use crate::located::{Located, located};

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
    let Located {
        node,
        prefix,
        index,
        ..
    } = located(&tree, after)?;
    let next = node
        .program
        .map
        .stages
        .get(index + 1)
        .ok_or_else(|| NextError::End(after.stage.to_owned()))?;
    if node.stages.contains_key(&next.id) {
        return Err(NextError::Taken(next.id.clone()));
    }
    let (leaf, next) = (node.program.clone(), next.clone());
    Ok(Ahead {
        tree,
        leaf,
        prefix,
        index,
        next,
    })
}
