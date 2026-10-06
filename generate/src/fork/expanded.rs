use std::fs;
use std::path::Path;

use tolearn_core::state::Lapse;

use crate::build::{Kit, guarded, replaced};
use crate::error::GenerateError;
use crate::plan;
use crate::sources::CACHE;
use crate::start::{Lineage, Opened, descend, lay, opened};
use crate::step::Step;

use super::landed::Landed;
use super::onward::Onward;

pub(super) fn expanded(
    kit: &mut Kit<'_>,
    onward: &Onward,
    folder: &Path,
    lapses: &[Lapse],
) -> Result<Landed, GenerateError> {
    let lineage = unfolded(kit, onward)?;
    let landed = written(kit, onward, &lineage, folder, lapses);
    if landed.is_err() {
        let cache = kit.data.join(CACHE);
        for program in lineage.programs() {
            let _ = fs::remove_dir_all(cache.join(&program.uuid));
        }
    }
    landed
}

fn unfolded(kit: &Kit<'_>, onward: &Onward) -> Result<Lineage, GenerateError> {
    let request = onward.request();
    let uuid = onward.row.uuid.clone();
    let tally = kit.online.tally();
    let mark = tally.len();
    let plan = guarded(kit.progress, kit.stop, Step::Part, || {
        plan::expand(kit.online, &request, &onward.part(), onward.depth)
    })?;
    tally.stamp(mark, &uuid, None);
    descend(kit, &request, &plan, uuid, onward.depth)
}

fn written(
    kit: &mut Kit<'_>,
    onward: &Onward,
    lineage: &Lineage,
    folder: &Path,
    lapses: &[Lapse],
) -> Result<Landed, GenerateError> {
    let Opened {
        lineage,
        stage,
        assets,
    } = opened(kit, lineage, lapses)?;
    let place = folder.join(&onward.prefix);
    replaced(kit, &onward.tree, folder, || {
        lay(&place, &lineage, &stage, &assets)
    })?;
    Ok(Landed {
        node: lineage.leaf.uuid,
        stage: stage.id,
    })
}
