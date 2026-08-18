use crate::generate::go;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{GenerateGoIn, GenerateGoOut};

pub fn run(_context: &Context, input: &GenerateGoIn) -> Result<GenerateGoOut, IpcError> {
    Ok(GenerateGoOut {
        going: go(&input.job),
    })
}
