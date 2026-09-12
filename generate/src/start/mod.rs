mod assemble;
mod cited;
mod day;
mod guarded;
mod kit;
mod lay;
mod lineage;

pub use day::day;
pub use kit::Kit;

use std::fs;
use std::path::Path;

use tolearn_core::library::Library;

use crate::error::GenerateError;
use crate::halt::{checked, sealed};
use crate::plan::{self, Flaw, Plan, Request};
use crate::sources::{CACHE, Sources};
use crate::stage::{self, Place};
use crate::step::Step;

use assemble::assemble;
use cited::cited;
use guarded::guarded;
use lay::lay;
use lineage::{Lineage, lineage};

const BUILD: &str = "build";

pub fn start(mut kit: Kit<'_>, request: &Request, plan: &Plan) -> Result<String, GenerateError> {
    let flaws = plan::check(plan, 1);
    if !flaws.is_empty() {
        return Err(unfit(&flaws));
    }
    let lineage = lineage(&kit, request, plan)?;
    let cache = kit.data.join(CACHE);
    let root = cache.join(&lineage.ancestors.first().unwrap_or(&lineage.leaf).uuid);
    let done = built(&mut kit, &lineage, &root.join(BUILD));
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

fn built(kit: &mut Kit<'_>, lineage: &Lineage, build: &Path) -> Result<String, GenerateError> {
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
    let (progress, stop, online) = (kit.progress, kit.stop, kit.online);
    let gathered = guarded(progress, stop, Step::Sources, || {
        let mut sources = Sources::new(
            kit.source,
            kit.renderer,
            &mut *kit.store,
            kit.data,
            &leaf.uuid,
            kit.at,
        );
        stage::gather(online, &mut sources, &place, stop)
    })?;
    checked(stop)?;
    let draft = stage::compose(online, &place, &gathered, progress)?;
    let (stage, assets) = guarded(progress, stop, Step::Diagrams, || {
        assemble(kit, draft.stage, &gathered)
    })?;
    let mut landed = lineage.clone();
    landed.leaf.sources = cited(&draft.cited, &gathered);
    guarded(progress, stop, Step::Write, || {
        lay(build, &landed, &stage, &assets)?;
        sealed(stop)?;
        Library::at(kit.data)
            .install(build)
            .map_err(GenerateError::Library)
    })
}

fn unfit(flaws: &[Flaw]) -> GenerateError {
    GenerateError::Unfit {
        flaws: flaws.iter().map(ToString::to_string).collect(),
    }
}
