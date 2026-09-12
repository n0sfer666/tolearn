use crate::REPAIRS;
use crate::error::GenerateError;
use crate::gate::Online;
use crate::plan::stage_fits;

use super::answer;
use super::draft::Draft;
use super::gathered::Gathered;
use super::mend::{self, Mending};
use super::part::parts;
use super::patch::patch;
use super::place::Place;
use super::prompt;
use super::raw;
use super::rules::invariants;

const WHAT: &str = "этап";

pub fn compose(
    online: &Online<'_>,
    place: &Place<'_>,
    gathered: &Gathered,
) -> Result<Draft, GenerateError> {
    if !stage_fits(place.row.hours) {
        return Err(GenerateError::StageHours {
            stage: place.row.id.clone(),
            hours: place.row.hours,
        });
    }
    let task = prompt::text(place, gathered);
    let mut prompt = task.clone();
    let mut open: Option<Mending> = None;
    let mut flaws = Vec::new();
    for _ in 0..=REPAIRS {
        let said = online.ask(&prompt)?.text;
        let read = match &open {
            None => raw::parse(&said),
            Some(open) => patch(open, &said),
        }
        .and_then(|raw| answer::build(&raw, place, gathered).map(|draft| (raw, draft)));
        match (read, open.take()) {
            (Ok((raw, draft)), _) => {
                flaws = invariants(&draft, place);
                if flaws.is_empty() {
                    return Ok(draft);
                }
                let next = Mending {
                    raw,
                    ids: draft
                        .stage
                        .every_block()
                        .map(|block| block.id.clone())
                        .collect(),
                    parts: parts(&flaws, &draft.stage),
                    flaws: flaws.clone(),
                };
                prompt = mend::parts(place, gathered, &next, &flaws);
                open = Some(next);
            }
            (Err(flaw), None) => {
                flaws = vec![flaw];
                prompt = mend::whole(&task, &said, &flaws);
            }
            (Err(flaw), Some(kept)) => {
                flaws = kept.flaws.iter().cloned().chain([flaw]).collect();
                prompt = mend::parts(place, gathered, &kept, &flaws);
                open = Some(kept);
            }
        }
    }
    Err(GenerateError::Unrepaired {
        what: WHAT,
        flaws: flaws.iter().map(ToString::to_string).collect(),
    })
}
