use tolearn_core::program::{StageRow, Tree};

pub fn row<'a>(tree: &'a Tree, node: &str, stage: &str) -> Option<&'a StageRow> {
    let branch = tree.branch(node)?;
    branch
        .tree
        .program
        .map
        .stages
        .iter()
        .find(|row| row.id == stage)
}
