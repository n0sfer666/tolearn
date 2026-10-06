use tolearn_core::state::State;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::examined::{AnswerIn, AnswerOut};
use crate::ipc::examining::{question, staged};
use crate::ipc::shelf;

pub fn run(context: &Context, input: &AnswerIn) -> Result<AnswerOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let question = question(stage, &input.question)?;
    let node = &branch.tree.program.uuid;
    let draft = State::update(context.data(), &tree.program.uuid, |state| {
        state.draft(node, &stage.id, &question.id, &input.text);
        state
            .drafts(node, &stage.id)
            .remove(&question.id)
            .unwrap_or_default()
    })?;
    Ok(AnswerOut { draft })
}
