mod assemble;
mod built;
mod cited;
mod day;
mod guarded;
mod kit;
mod merged;
mod saved;
mod settled;
mod staged;
mod swapped;

pub use day::day;
pub use kit::Kit;

use tolearn_core::stage::Stage;

use crate::error::GenerateError;
use crate::halt::checked;
use crate::sources::Sources;
use crate::stage::{self, Gathered, Place};
use crate::step::Step;

pub(crate) use assemble::Assets;
use assemble::assemble;
pub(crate) use built::Built;
use cited::cited;
pub(crate) use guarded::guarded;
pub(crate) use settled::settled;
pub(crate) use staged::{described, staged};
pub(crate) use swapped::{Swap, replaced, swapped};

pub(crate) const BUILD: &str = "build";

pub(crate) fn build(kit: &mut Kit<'_>, place: &Place<'_>) -> Result<Built, GenerateError> {
    built(kit, place, None)
}

pub(crate) fn rebuild(
    kit: &mut Kit<'_>,
    place: &Place<'_>,
    previous: &Stage,
) -> Result<Built, GenerateError> {
    built(kit, place, Some(previous))
}

fn built(
    kit: &mut Kit<'_>,
    place: &Place<'_>,
    previous: Option<&Stage>,
) -> Result<Built, GenerateError> {
    let (progress, stop, online) = (kit.progress, kit.stop, kit.online);
    let tally = online.tally();
    let mark = tally.len();
    let kept = previous.and_then(|_| saved::load(kit.data, &place.program.uuid, place.row));
    let gathered = match kept {
        Some(gathered) => gathered,
        None => gather(kit, place)?,
    };
    checked(stop)?;
    let draft = match previous {
        Some(previous) => stage::recompose(online, place, &gathered, previous, progress)?,
        None => stage::compose(online, place, &gathered, progress)?,
    };
    tally.stamp(mark, &place.program.uuid, Some(&place.row.id));
    let (stage, assets) = guarded(progress, stop, Step::Diagrams, || {
        assemble(kit, draft.stage, &gathered)
    })?;
    Ok(Built {
        stage,
        assets,
        cited: cited(&draft.cited, &gathered),
    })
}

fn gather(kit: &mut Kit<'_>, place: &Place<'_>) -> Result<Gathered, GenerateError> {
    let (progress, stop, online) = (kit.progress, kit.stop, kit.online);
    let uuid = &place.program.uuid;
    guarded(progress, stop, Step::Sources, || {
        let mut sources = Sources::new(
            kit.source,
            kit.renderer,
            &mut *kit.store,
            kit.data,
            uuid,
            kit.at,
        )
        .counted(online.tally());
        let gathered = stage::gather(online, &mut sources, place, stop)?;
        let _ = saved::save(kit.data, uuid, place.row, &gathered);
        Ok(gathered)
    })
}
