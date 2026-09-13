use tolearn_core::state::{Attempt, Sitting, State};
use tolearn_core::verdict;
use tolearn_generate::start::day;

use crate::ipc::clock::now;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::examined::{ExamOut, ExamPasteIn};
use crate::ipc::examining::staged;
use crate::ipc::shelf;

pub fn run(context: &Context, input: &ExamPasteIn) -> Result<ExamOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let per_question = verdict::read(&input.text, stage)
        .map_err(|error| IpcError::new("exam.verdict", error.to_string()))?;
    let attempt = Attempt {
        on: day(now()),
        by: Sitting::Copypaste,
        model: None,
        per_question,
    };
    let passed = attempt.passes();
    let node = &branch.tree.program.uuid;
    State::update(context.data(), &tree.program.uuid, |state| {
        state.attempt(node, &stage.id, attempt);
    })?;
    Ok(ExamOut { passed })
}
