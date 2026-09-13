use std::path::Path;

use tolearn_core::program::StageRow;
use tolearn_core::state::Lapse;

use crate::build::{BUILD, Kit, Swap, build, settled, swapped};
use crate::error::GenerateError;
use crate::sources::CACHE;
use crate::stage::Place;

use super::after::After;
use super::ahead::{Ahead, Road};
use super::error::NextError;
use super::expanded::expanded;
use super::kept;
use super::landed::Landed;
use super::looked::{Looked, looked};

pub fn take(
    mut kit: Kit<'_>,
    after: &After<'_>,
    choice: usize,
    lapses: &[Lapse],
) -> Result<Landed, GenerateError> {
    let Looked::Ready(road, fork) = looked(kit.data, after)? else {
        return Err(NextError::Unforked(after.stage.to_owned()).into());
    };
    let count = fork.variants.len();
    let variant = fork
        .variants
        .into_iter()
        .nth(choice)
        .ok_or(NextError::Choice { choice, count })?;
    let folder = kit.data.join(CACHE).join(after.program).join(BUILD);
    let landed = match road {
        Road::Stage(ahead) => in_place(&mut kit, &ahead, variant.row, &folder, lapses),
        Road::Part(onward) => expanded(&mut kit, &onward, &folder, lapses),
    };
    if landed.is_ok() {
        let _ = kept::forget(kit.data, after);
    }
    settled(&kit, after.program, after.node, &folder, &landed);
    landed
}

fn in_place(
    kit: &mut Kit<'_>,
    ahead: &Ahead,
    row: StageRow,
    folder: &Path,
    lapses: &[Lapse],
) -> Result<Landed, GenerateError> {
    let id = row.id.clone();
    let mut leaf = ahead.leaf.clone();
    if let Some(slot) = leaf.map.stages.get_mut(ahead.index + 1) {
        *slot = row;
    }
    let found = Place::find(&leaf, &id).ok_or_else(|| NextError::Stage(id.clone()))?;
    let place = Place { lapses, ..found };
    let built = build(kit, &place)?;
    let swap = Swap {
        tree: &ahead.tree,
        prefix: &ahead.prefix,
        folder,
    };
    swapped(kit, &swap, &leaf, built)?;
    Ok(Landed {
        node: leaf.uuid,
        stage: id,
    })
}
