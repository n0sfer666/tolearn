mod lay;
mod lineage;

pub use crate::build::{Kit, day};

use std::fs;
use std::path::Path;

use tolearn_core::library::Library;

use crate::build::{BUILD, build, guarded};
use crate::error::GenerateError;
use crate::halt::sealed;
use crate::ledger;
use crate::plan::{self, Flaw, Plan, Request};
use crate::sources::CACHE;
use crate::stage::Place;
use crate::step::Step;

use lay::lay;
use lineage::{Lineage, lineage};

pub fn start(mut kit: Kit<'_>, request: &Request, plan: &Plan) -> Result<String, GenerateError> {
    let flaws = plan::check(plan, 1);
    if !flaws.is_empty() {
        return Err(unfit(&flaws));
    }
    let lineage = lineage(&kit, request, plan)?;
    let cache = kit.data.join(CACHE);
    let root = cache.join(&lineage.root().uuid);
    let done = first(&mut kit, &lineage, &root.join(BUILD));
    let _ = fs::remove_dir_all(root.join(BUILD));
    if done.is_err() {
        for program in lineage.programs() {
            let _ = fs::remove_dir_all(cache.join(&program.uuid));
        }
    } else if !lineage.ancestors.is_empty() {
        let _ = fs::remove_dir(&root);
    }
    done
}

fn first(kit: &mut Kit<'_>, lineage: &Lineage, folder: &Path) -> Result<String, GenerateError> {
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
    };
    let built = build(kit, &place)?;
    let mut landed = lineage.clone();
    landed.leaf.sources = built.cited;
    let (stop, tally) = (kit.stop, kit.online.tally());
    guarded(kit.progress, stop, Step::Write, || {
        lay(folder, &landed, &built.stage, &built.assets)?;
        let root = &lineage.root().uuid;
        tally.stamp(0, root, None);
        tally.date(kit.at);
        ledger::append(&ledger::path(kit.data, root), &tally.records())?;
        sealed(stop)?;
        Library::at(kit.data)
            .install(folder)
            .map_err(GenerateError::Library)
    })
}

fn unfit(flaws: &[Flaw]) -> GenerateError {
    GenerateError::Unfit {
        flaws: flaws.iter().map(ToString::to_string).collect(),
    }
}
