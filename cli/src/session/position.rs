use tolearn_core::program::{StageRow, Tree};

#[derive(Debug, Clone, Copy)]
pub struct Position<'a> {
    pub node: &'a Tree,
    pub row: &'a StageRow,
}

pub fn position(tree: &Tree) -> Option<Position<'_>> {
    let deeper = tree
        .program
        .map
        .children
        .iter()
        .rev()
        .filter_map(|row| {
            tree.children
                .values()
                .find(|child| child.program.uuid == row.uuid)
        })
        .find_map(position);
    deeper.or_else(|| {
        tree.program
            .map
            .stages
            .iter()
            .rev()
            .find(|row| tree.stages.contains_key(&row.id))
            .map(|row| Position { node: tree, row })
    })
}

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
