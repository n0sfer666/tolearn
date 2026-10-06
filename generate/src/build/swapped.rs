use std::fs;
use std::path::Path;

use tolearn_core::library::Library;
use tolearn_core::program::{Program, Tree};

use crate::error::GenerateError;
use crate::halt::sealed;
use crate::step::Step;

use super::built::Built;
use super::guarded::guarded;
use super::kit::Kit;
use super::merged::merged;
use super::staged::staged;

pub(crate) struct Swap<'a> {
    pub(crate) tree: &'a Tree,
    pub(crate) prefix: &'a str,
    pub(crate) folder: &'a Path,
}

pub(crate) fn swapped(
    kit: &Kit<'_>,
    swap: &Swap<'_>,
    leaf: &Program,
    built: Built,
) -> Result<(), GenerateError> {
    let Built {
        stage,
        assets,
        cited,
    } = built;
    let landed = Program {
        sources: merged(leaf.sources.clone(), cited),
        ..leaf.clone()
    };
    replaced(kit, swap.tree, swap.folder, || {
        staged(&swap.folder.join(swap.prefix), &landed, &stage, &assets)
    })
}

pub(crate) fn replaced(
    kit: &Kit<'_>,
    tree: &Tree,
    folder: &Path,
    put: impl FnOnce() -> Result<(), GenerateError>,
) -> Result<(), GenerateError> {
    guarded(kit.progress, kit.stop, Step::Write, || {
        let library = Library::at(kit.data);
        let _ = fs::remove_dir_all(folder);
        library.copy(tree, folder).map_err(GenerateError::Library)?;
        put()?;
        sealed(kit.stop)?;
        library.replace(folder).map_err(GenerateError::Library)
    })?;
    Ok(())
}
