mod doubt;
mod prompt;

pub use doubt::Doubt;
pub use prompt::prompt;

use std::path::Path;

use crate::error::GenerateError;
use crate::gate::Online;
use crate::ledger;
use crate::step::Step;

pub const CLARIFY_PROMPT_CHARS: usize = 18_000;
pub const BLOCK_CHARS: usize = 4_000;
pub const CHAIN_CHARS: usize = 6_000;
pub const QUESTION_CHARS: usize = 1_000;
pub const ANSWER_CHARS: usize = 1_500;

pub fn clarify(
    online: &Online<'_>,
    data: &Path,
    doubt: &Doubt<'_>,
    at: i64,
) -> Result<String, GenerateError> {
    let tally = online.tally();
    let said = online.ask(Step::Clarify, &prompt(doubt));
    tally.stamp(0, doubt.node, Some(&doubt.place.row.id));
    tally.date(at);
    let _ = ledger::append(&ledger::path(data, doubt.program), &tally.take());
    let answer = said?.text.trim().to_owned();
    if answer.is_empty() {
        return Err(GenerateError::Unclear);
    }
    Ok(answer)
}
