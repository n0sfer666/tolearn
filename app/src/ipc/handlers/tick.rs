use tolearn_core::state::State;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::practice::claim;
use crate::ipc::practiced::{TickIn, TickOut};
use crate::ipc::shelf;

pub fn run(context: &Context, input: &TickIn) -> Result<TickOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let check = claim(branch.tree, &input.stage, &input.claim)?;
    let node = &branch.tree.program.uuid;
    let ticks = State::update(context.data(), &tree.program.uuid, |state| {
        state.tick(node, &input.stage, &check.id, input.on);
        state.ticks(node, &input.stage)
    })?;
    Ok(TickOut { ticks })
}
