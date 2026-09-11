use std::path::Path;

use super::refusal::Refusal;
use crate::program::{self, Tree};

pub(super) fn read(directory: &Path) -> Result<Tree, Refusal> {
    let tree = program::load(directory).map_err(Refusal::Unloadable)?;
    let violations = program::validate(&tree);
    if violations.is_empty() {
        Ok(tree)
    } else {
        Err(Refusal::Invalid(violations))
    }
}

pub(super) fn examine(directory: &Path, name: &str) -> Result<Tree, Refusal> {
    let tree = read(directory)?;
    if tree.program.uuid != name {
        return Err(Refusal::Misplaced {
            directory: name.to_owned(),
            uuid: tree.program.uuid,
        });
    }
    Ok(tree)
}
