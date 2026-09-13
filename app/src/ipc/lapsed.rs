use tolearn_core::state::{Lapse, State};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;

pub fn lapses(context: &Context, program: &str) -> Result<Vec<Lapse>, IpcError> {
    let tree = context.library().open(program)?;
    Ok(State::read(context.data(), &tree.program.uuid)?.lapses(&tree))
}
