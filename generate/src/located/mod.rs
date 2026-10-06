mod missing;

pub(crate) use missing::Missing;

use tolearn_core::program::Tree;
use tolearn_core::stage::Stage;

use crate::fork::After;

pub(crate) struct Located<'a> {
    pub(crate) node: &'a Tree,
    pub(crate) prefix: String,
    pub(crate) index: usize,
    pub(crate) stage: &'a Stage,
}

pub(crate) fn located<'a>(tree: &'a Tree, after: &After<'_>) -> Result<Located<'a>, Missing> {
    let branch = tree
        .branch(after.node)
        .ok_or_else(|| Missing::Node(after.node.to_owned()))?;
    let node = branch.tree;
    let index = node
        .program
        .map
        .stages
        .iter()
        .position(|row| row.id == after.stage)
        .ok_or_else(|| Missing::Stage(after.stage.to_owned()))?;
    let stage = node
        .stages
        .get(after.stage)
        .ok_or_else(|| Missing::Ungenerated(after.stage.to_owned()))?;
    Ok(Located {
        node,
        prefix: branch.prefix,
        index,
        stage,
    })
}
