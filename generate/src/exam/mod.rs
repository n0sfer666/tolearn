mod paper;
mod prompt;

pub use paper::Paper;
pub use prompt::prompt;

use std::path::Path;

use tolearn_core::state::Answered;
use tolearn_core::verdict::{self, VerdictError};

use crate::error::GenerateError;
use crate::gate::Online;
use crate::ledger;
use crate::step::Step;

const MENDS: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sat {
    pub per_question: Vec<Answered>,
    pub model: Option<String>,
}

pub fn sit(
    online: &Online<'_>,
    data: &Path,
    paper: &Paper<'_>,
    at: i64,
) -> Result<Sat, GenerateError> {
    let tally = online.tally();
    let sat = graded(online, paper);
    tally.stamp(0, paper.node, Some(&paper.stage.id));
    tally.date(at);
    let _ = ledger::append(&ledger::path(data, paper.program), &tally.take());
    sat
}

fn graded(online: &Online<'_>, paper: &Paper<'_>) -> Result<Sat, GenerateError> {
    let task = prompt(paper);
    let mut asked = task.clone();
    let mut error = VerdictError::Absent;
    for round in 0..=MENDS {
        let said = match round {
            0 => online.ask(Step::Exam, &asked)?,
            round => online.again(Step::Exam, round, &asked)?,
        };
        match verdict::read(&said.text, paper.stage) {
            Ok(per_question) => {
                return Ok(Sat {
                    per_question,
                    model: said.model,
                });
            }
            Err(wrong) => {
                asked = prompt::repair(&task, &said.text, &wrong);
                error = wrong;
            }
        }
    }
    Err(GenerateError::Verdict(error))
}
