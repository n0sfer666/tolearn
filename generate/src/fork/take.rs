use std::path::Path;

use tolearn_core::program::StageRow;

use crate::build::{BUILD, Kit, Swap, build, settled, swapped};
use crate::error::GenerateError;
use crate::sources::CACHE;
use crate::stage::Place;

use super::after::After;
use super::ahead::Ahead;
use super::error::NextError;
use super::{kept, looked};

pub fn take(mut kit: Kit<'_>, after: &After<'_>, choice: usize) -> Result<String, GenerateError> {
    let (ahead, kept) = looked(kit.data, after)?;
    let fork = kept.ok_or_else(|| NextError::Unforked(after.stage.to_owned()))?;
    let count = fork.variants.len();
    let variant = fork
        .variants
        .into_iter()
        .nth(choice)
        .ok_or(NextError::Choice { choice, count })?;
    let folder = kit.data.join(CACHE).join(after.program).join(BUILD);
    let landed = landed(&mut kit, &ahead, variant.row, &folder);
    if landed.is_ok() {
        let _ = kept::forget(kit.data, after);
    }
    settled(&kit, after.program, after.node, &folder, &landed);
    landed
}

fn landed(
    kit: &mut Kit<'_>,
    ahead: &Ahead,
    row: StageRow,
    folder: &Path,
) -> Result<String, GenerateError> {
    let id = row.id.clone();
    let mut leaf = ahead.leaf.clone();
    if let Some(slot) = leaf.map.stages.get_mut(ahead.index + 1) {
        *slot = row;
    }
    let place = Place::find(&leaf, &id).ok_or_else(|| NextError::Stage(id.clone()))?;
    let built = build(kit, &place)?;
    let swap = Swap {
        tree: &ahead.tree,
        prefix: &ahead.prefix,
        folder,
    };
    swapped(kit, &swap, &leaf, built)?;
    Ok(id)
}
