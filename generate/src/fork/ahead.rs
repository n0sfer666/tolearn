use tolearn_core::program::{Program, StageRow, Tree};

use crate::located::{Located, located};

use super::after::After;
use super::error::NextError;
use super::onward::{Onward, onward};

pub(super) enum Road {
    Stage(Ahead),
    Part(Onward),
}

#[derive(Debug, Clone)]
pub(super) struct Ahead {
    pub(super) tree: Tree,
    pub(super) leaf: Program,
    pub(super) prefix: String,
    pub(super) index: usize,
    pub(super) next: StageRow,
}

pub(super) fn ahead(tree: Tree, after: &After<'_>) -> Result<Road, NextError> {
    let Located {
        node,
        prefix,
        index,
        ..
    } = located(&tree, after)?;
    let Some(next) = node.program.map.stages.get(index + 1) else {
        return onward(tree, after).map(Road::Part);
    };
    if node.stages.contains_key(&next.id) {
        return Err(NextError::Taken(next.id.clone()));
    }
    let (leaf, next) = (node.program.clone(), next.clone());
    Ok(Road::Stage(Ahead {
        tree,
        leaf,
        prefix,
        index,
        next,
    }))
}
