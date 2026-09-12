use tolearn_core::program::{ChildRow, Generation, Map, Program, Sources};

use crate::error::GenerateError;
use crate::plan::{self, Plan, Request};
use crate::step::Step;

use super::guarded::guarded;
use super::kit::Kit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Lineage {
    pub(super) ancestors: Vec<Program>,
    pub(super) leaf: Program,
}

impl Lineage {
    pub(super) fn root(&self) -> &Program {
        self.ancestors.first().unwrap_or(&self.leaf)
    }

    pub(super) fn programs(&self) -> impl Iterator<Item = &Program> {
        self.ancestors.iter().chain([&self.leaf])
    }
}

pub(super) fn lineage(
    kit: &Kit<'_>,
    request: &Request,
    plan: &Plan,
) -> Result<Lineage, GenerateError> {
    let mut ancestors = Vec::new();
    let mut leaf = program(plan, request, fresh());
    let mut current = plan.clone();
    while let (Some(part), Some(row)) = (current.children.first(), leaf.map.children.first()) {
        let uuid = row.uuid.clone();
        let depth = ancestors.len() + 2;
        let mark = kit.online.tally().len();
        let next = guarded(kit.progress, kit.stop, Step::Part, || {
            plan::expand(kit.online, request, part, depth)
        })?;
        kit.online.tally().stamp(mark, &uuid, None);
        ancestors.push(std::mem::replace(&mut leaf, program(&next, request, uuid)));
        current = next;
    }
    Ok(Lineage { ancestors, leaf })
}

fn program(plan: &Plan, request: &Request, uuid: String) -> Program {
    Program {
        uuid,
        slug: plan.slug.clone(),
        title: plan.title.clone(),
        goal: plan.goal.clone(),
        level: request.level.clone(),
        generation: Generation {
            locale: request.locale.clone(),
            volatility: plan.volatility,
            request: request.request.clone(),
        },
        map: Map {
            stages: plan.stages.clone(),
            children: plan
                .children
                .iter()
                .map(|part| ChildRow {
                    uuid: fresh(),
                    title: part.title.clone(),
                    hours: part.hours,
                })
                .collect(),
        },
        sources: Sources {
            books: Vec::new(),
            pages: Vec::new(),
        },
    }
}

fn fresh() -> String {
    uuid::Uuid::new_v4().to_string()
}
