use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{LlmLogIn, LlmLogOut};
use crate::journal;

pub fn run(context: &Context, input: &LlmLogIn) -> Result<LlmLogOut, IpcError> {
    let room = context.llm_log();
    if input.clear {
        journal::clear(&room);
    }
    if input.open {
        journal::reveal(&room).map_err(|reason| {
            IpcError::new(
                "journal.unopened",
                format!("папка журнала не открылась: {reason}"),
            )
        })?;
    }
    Ok(LlmLogOut {
        room: room.display().to_string(),
        records: u32::try_from(journal::records(&room).len()).unwrap_or(u32::MAX),
    })
}
