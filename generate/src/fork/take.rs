use std::fs;
use std::path::Path;

use tolearn_core::library::Library;
use tolearn_core::program::{Program, StageRow};

use crate::build::{BUILD, Kit, build, guarded, staged};
use crate::error::GenerateError;
use crate::halt::sealed;
use crate::ledger;
use crate::sources::CACHE;
use crate::stage::Place;
use crate::step::Step;

use super::after::After;
use super::ahead::Ahead;
use super::error::NextError;
use super::merged::merged;
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
    let _ = fs::remove_dir_all(&folder);
    let tally = kit.online.tally();
    if landed.is_ok() {
        let _ = kept::forget(kit.data, after);
    } else {
        tally.extend(tally.release());
    }
    tally.stamp(0, after.node, None);
    tally.date(kit.at);
    let _ = ledger::append(&ledger::path(kit.data, after.program), &tally.take());
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
    let landed = Program {
        sources: merged(leaf.sources.clone(), built.cited),
        ..leaf.clone()
    };
    guarded(kit.progress, kit.stop, Step::Write, || {
        let library = Library::at(kit.data);
        let _ = fs::remove_dir_all(folder);
        library
            .copy(&ahead.tree, folder)
            .map_err(GenerateError::Library)?;
        staged(
            &folder.join(&ahead.prefix),
            &landed,
            &built.stage,
            &built.assets,
        )?;
        sealed(kit.stop)?;
        library.replace(folder).map_err(GenerateError::Library)
    })?;
    Ok(id)
}
