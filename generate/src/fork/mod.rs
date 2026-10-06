mod after;
mod ahead;
mod answer;
mod error;
mod expanded;
mod flaw;
mod kept;
mod landed;
mod looked;
mod offered;
mod onward;
mod prompt;
mod rules;
mod take;
mod variant;

pub use after::After;
pub use error::NextError;
pub use landed::Landed;
pub use offered::Fork;
pub use take::take;
pub use variant::Variant;

use std::path::Path;

use tolearn_core::state::Lapse;

use crate::REPAIRS;
use crate::error::GenerateError;
use crate::gate::Online;
use crate::ledger;
use crate::step::Step;

use ahead::Ahead;
use looked::{Looked, looked};

pub const MAX_ALTERNATIVES: usize = 2;

const WHAT: &str = "развилку";

pub fn known(data: &Path, after: &After<'_>) -> Result<Option<Fork>, GenerateError> {
    Ok(match looked(data, after)? {
        Looked::Ready(_, fork) => Some(fork),
        Looked::Open(_) => None,
    })
}

pub fn propose(
    online: &Online<'_>,
    data: &Path,
    after: &After<'_>,
    at: i64,
    lapses: &[Lapse],
) -> Result<Fork, GenerateError> {
    let ahead = match looked(data, after)? {
        Looked::Ready(_, fork) => return Ok(fork),
        Looked::Open(ahead) => ahead,
    };
    let tally = online.tally();
    let drawn =
        draw(online, &ahead, lapses).and_then(|fork| kept::save(data, after, &fork).map(|()| fork));
    tally.stamp(0, after.node, None);
    tally.date(at);
    let _ = ledger::append(&ledger::path(data, after.program), &tally.take());
    drawn
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
