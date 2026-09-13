use tolearn_core::state::State;
use tolearn_generate::start::day;

use crate::ipc::clock::now;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::examined::SkipOut;
use crate::ipc::examining::staged;
use crate::ipc::reading::StageIn;
use crate::ipc::shelf;

pub fn run(context: &Context, input: &StageIn) -> Result<SkipOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let node = &branch.tree.program.uuid;
    let today = day(now());
    let skipped = State::update(context.data(), &tree.program.uuid, |state| {
        state.skip(node, &stage.id, &today)
    })?;
    Ok(SkipOut { skipped })
}
