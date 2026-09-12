mod answer;
mod flaw;
mod prompt;
mod rules;
mod shown;
mod types;

pub use flaw::Flaw;
pub(crate) use flaw::span;
pub use rules::check;
pub use types::{Part, Plan, Request};

use tolearn_core::Hours;

use crate::REPAIRS;
use crate::error::GenerateError;
use crate::gate::Online;
use crate::step::Step;

pub const STAGE_MIN_HOURS: u32 = 2;
pub const STAGE_MAX_HOURS: u32 = 4;
pub const MAX_HOURS: u32 = 70;
pub const MAX_STAGES: usize = 25;

pub fn stage_fits(hours: Hours) -> bool {
    STAGE_MIN_HOURS <= hours.min && hours.min <= hours.max && hours.max <= STAGE_MAX_HOURS
}

const WHAT: &str = "карту";

pub fn plan(online: &Online<'_>, request: &Request) -> Result<Plan, GenerateError> {
    draw(online, Step::Plan, &prompt::task(request, None, 1), 1)
}

pub fn expand(
    online: &Online<'_>,
    request: &Request,
    part: &Part,
    depth: usize,
) -> Result<Plan, GenerateError> {
    draw(
        online,
        Step::Part,
        &prompt::task(request, Some(part), depth),
        depth,
    )
}

pub fn revise(
    online: &Online<'_>,
    request: &Request,
    previous: &Plan,
    wish: &str,
) -> Result<Plan, GenerateError> {
    draw(
        online,
        Step::Revise,
        &prompt::revised(request, previous, wish),
        1,
    )
}

fn draw(online: &Online<'_>, step: Step, task: &str, depth: usize) -> Result<Plan, GenerateError> {
    let mut prompt = task.to_owned();
    let mut flaws = Vec::new();
    for round in 0..=REPAIRS {
        let said = match round {
            0 => online.ask(step, &prompt)?,
            round => online.again(step, round, &prompt)?,
        };
        flaws = match answer::read(&said.text) {
            Ok(plan) => {
                let flaws = check(&plan, depth);
                if flaws.is_empty() {
                    return Ok(plan);
                }
                flaws
            }
            Err(flaw) => vec![flaw],
        };
        prompt = prompt::repair(task, &said.text, &flaws);
    }
    Err(GenerateError::Unrepaired {
        what: WHAT,
        flaws: flaws.iter().map(ToString::to_string).collect(),
    })
}
