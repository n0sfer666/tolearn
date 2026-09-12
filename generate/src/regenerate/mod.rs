mod error;
mod found;

pub use error::RegenerateError;

use std::path::Path;

use tolearn_core::library::Library;

use crate::build::{BUILD, Kit, Swap, rebuild, settled, swapped};
use crate::error::GenerateError;
use crate::fork::After;
use crate::sources::CACHE;
use crate::stage::Place;

use found::{Found, found};

pub fn regenerate(mut kit: Kit<'_>, at: &After<'_>) -> Result<(), GenerateError> {
    let tree = Library::at(kit.data)
        .open(at.program)
        .map_err(GenerateError::Library)?;
    let found = found(tree, at)?;
    let folder = kit.data.join(CACHE).join(at.program).join(BUILD);
    let landed = landed(&mut kit, &found, &folder);
    settled(&kit, at.program, at.node, &folder, &landed);
    landed
}

fn landed(kit: &mut Kit<'_>, found: &Found, folder: &Path) -> Result<(), GenerateError> {
    let id = &found.previous.id;
    let place = Place::find(&found.leaf, id).ok_or_else(|| RegenerateError::Stage(id.clone()))?;
    let built = rebuild(kit, &place, &found.previous)?;
    let swap = Swap {
        tree: &found.tree,
        prefix: &found.prefix,
        folder,
    };
    swapped(kit, &swap, &found.leaf, built)
}
