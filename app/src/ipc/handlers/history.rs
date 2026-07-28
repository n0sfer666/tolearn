use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::history::versions;
use crate::ipc::open;
use crate::ipc::types::{HistoryIn, HistoryOut, VersionView};

pub fn run(context: &Context, input: &HistoryIn) -> Result<HistoryOut, IpcError> {
    let scan = open::read(&input.bundle)?;

    Ok(HistoryOut {
        versions: versions(&context.history(&scan.roadmap.id))?
            .iter()
            .map(|kept| VersionView {
                n: kept.n,
                saved_at: kept.saved_at.clone(),
                bytes: kept.bytes,
            })
            .collect(),
    })
}
