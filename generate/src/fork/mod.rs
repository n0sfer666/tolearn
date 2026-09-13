mod after;
mod ahead;
mod answer;
mod error;
mod flaw;
mod kept;
mod offered;
mod prompt;
mod rules;
mod take;
mod variant;

pub use after::After;
pub use error::NextError;
pub use offered::Fork;
pub use take::take;
pub use variant::Variant;

use std::path::Path;

use tolearn_core::library::Library;
use tolearn_core::state::Lapse;

use crate::REPAIRS;
use crate::error::GenerateError;
use crate::gate::Online;
use crate::ledger;
use crate::step::Step;

use ahead::{Ahead, ahead};

pub const MAX_ALTERNATIVES: usize = 2;

const WHAT: &str = "развилку";

pub fn known(data: &Path, after: &After<'_>) -> Result<Option<Fork>, GenerateError> {
    looked(data, after).map(|(_, fork)| fork)
}

pub fn propose(
    online: &Online<'_>,
    data: &Path,
    after: &After<'_>,
    at: i64,
    lapses: &[Lapse],
) -> Result<Fork, GenerateError> {
    let (ahead, kept) = looked(data, after)?;
    if let Some(fork) = kept {
        return Ok(fork);
    }
    let tally = online.tally();
    let drawn =
        draw(online, &ahead, lapses).and_then(|fork| kept::save(data, after, &fork).map(|()| fork));
    tally.stamp(0, after.node, None);
    tally.date(at);
    let _ = ledger::append(&ledger::path(data, after.program), &tally.take());
    drawn
}

fn looked(data: &Path, after: &After<'_>) -> Result<(Ahead, Option<Fork>), GenerateError> {
    let tree = Library::at(data)
        .open(after.program)
        .map_err(GenerateError::Library)?;
    let ahead = ahead(tree, after)?;
    let fork = kept::load(data, after)?.filter(|fork| fresh(fork, &ahead));
    Ok((ahead, fork))
}

fn fresh(fork: &Fork, ahead: &Ahead) -> bool {
    fork.variants
        .first()
        .is_some_and(|variant| variant.row == ahead.next)
        && rules::check(fork, &ahead.leaf.map).is_empty()
}

fn draw(online: &Online<'_>, ahead: &Ahead, lapses: &[Lapse]) -> Result<Fork, GenerateError> {
    let task = prompt::task(ahead, lapses);
    let mut prompt = task.clone();
    let mut flaws = Vec::new();
    for round in 0..=REPAIRS {
        let said = match round {
            0 => online.ask(Step::Fork, &prompt)?,
            round => online.again(Step::Fork, round, &prompt)?,
        };
        flaws = match answer::read(&said.text, &ahead.next) {
            Ok(fork) => {
                let flaws = rules::check(&fork, &ahead.leaf.map);
                if flaws.is_empty() {
                    return Ok(fork);
                }
                flaws
            }
            Err(flaw) => vec![flaw],
        };
        prompt = prompt::repair(&task, &said.text, &flaws);
    }
    Err(GenerateError::Unrepaired {
        what: WHAT,
        flaws: flaws.iter().map(ToString::to_string).collect(),
    })
}
