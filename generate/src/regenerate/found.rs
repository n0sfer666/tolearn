use tolearn_core::program::{Program, Tree};
use tolearn_core::stage::Stage;

use crate::fork::After;
use crate::located::{Located, located};

use super::error::RegenerateError;

pub(super) struct Found {
    pub(super) tree: Tree,
    pub(super) leaf: Program,
    pub(super) prefix: String,
    pub(super) previous: Stage,
}

pub(super) fn found(tree: Tree, at: &After<'_>) -> Result<Found, RegenerateError> {
    let Located {
        node,
        prefix,
        stage,
        ..
    } = located(&tree, at)?;
    let (leaf, previous) = (node.program.clone(), stage.clone());
    Ok(Found {
        tree,
        leaf,
        prefix,
        previous,
    })
}
