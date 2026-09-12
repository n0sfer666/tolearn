mod assemble;
mod built;
mod cited;
mod day;
mod guarded;
mod kit;
mod staged;

pub use day::day;
pub use kit::Kit;

use crate::error::GenerateError;
use crate::halt::checked;
use crate::sources::Sources;
use crate::stage::{self, Place};
use crate::step::Step;

pub(crate) use assemble::Assets;
use assemble::assemble;
pub(crate) use built::Built;
use cited::cited;
pub(crate) use guarded::guarded;
pub(crate) use staged::{described, staged};

pub(crate) const BUILD: &str = "build";

pub(crate) fn build(kit: &mut Kit<'_>, place: &Place<'_>) -> Result<Built, GenerateError> {
    let (progress, stop, online) = (kit.progress, kit.stop, kit.online);
    let tally = online.tally();
    let mark = tally.len();
    let uuid = &place.program.uuid;
    let gathered = guarded(progress, stop, Step::Sources, || {
        let mut sources = Sources::new(
            kit.source,
            kit.renderer,
            &mut *kit.store,
            kit.data,
            uuid,
            kit.at,
        )
        .counted(tally);
        stage::gather(online, &mut sources, place, stop)
    })?;
    checked(stop)?;
    let draft = stage::compose(online, place, &gathered, progress)?;
    tally.stamp(mark, uuid, Some(&place.row.id));
    let (stage, assets) = guarded(progress, stop, Step::Diagrams, || {
        assemble(kit, draft.stage, &gathered)
    })?;
    Ok(Built {
        stage,
        assets,
        cited: cited(&draft.cited, &gathered),
    })
}
