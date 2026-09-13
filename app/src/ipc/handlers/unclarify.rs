use crate::ipc::clarified::{ChainIn, ClarificationsOut};
use crate::ipc::clarifying::chained;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;

pub fn run(context: &Context, input: &ChainIn) -> Result<ClarificationsOut, IpcError> {
    chained(context, input, |state, node, stage, index| {
        state.unclarify(node, stage, index)
    })
}
