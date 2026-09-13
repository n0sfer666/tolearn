use tolearn_core::stage::Stage;
use tolearn_core::state::Lapse;

use crate::build::{Assets, Kit, build};
use crate::error::GenerateError;
use crate::plan::Flaw;
use crate::stage::Place;

use super::lineage::Lineage;
use super::unfit;

pub(crate) struct Opened {
    pub(crate) lineage: Lineage,
    pub(crate) stage: Stage,
    pub(crate) assets: Assets,
}

pub(crate) fn opened(
    kit: &mut Kit<'_>,
    lineage: &Lineage,
    lapses: &[Lapse],
) -> Result<Opened, GenerateError> {
    let leaf = &lineage.leaf;
    let row = leaf
        .map
        .stages
        .first()
        .ok_or_else(|| unfit(&[Flaw::Empty]))?;
    let place = Place {
        program: leaf,
        row,
        index: 0,
        lapses,
    };
    let built = build(kit, &place)?;
    let mut landed = lineage.clone();
    landed.leaf.sources = built.cited;
    Ok(Opened {
        lineage: landed,
        stage: built.stage,
        assets: built.assets,
    })
}
