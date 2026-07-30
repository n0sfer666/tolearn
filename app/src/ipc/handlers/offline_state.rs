use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{LeftView, OfflineStateIn, OfflineStateOut};
use crate::offline;

pub fn run(_context: &Context, input: &OfflineStateIn) -> Result<OfflineStateOut, IpcError> {
    let live = offline::look(&input.job).ok_or_else(|| unknown(&input.job))?;
    Ok(OfflineStateOut {
        total: count(live.total),
        done: count(live.done),
        current: live.current,
        finished: live.finished,
        cancelled: live.cancelled,
        bytes: live.bytes,
        saved: live.saved,
        skipped: live.skipped.into_iter().map(left).collect(),
        failed: live.failed.into_iter().map(left).collect(),
    })
}

fn left((url, why): (String, String)) -> LeftView {
    LeftView { url, why }
}

fn count(many: usize) -> u32 {
    u32::try_from(many).unwrap_or(u32::MAX)
}

fn unknown(job: &str) -> IpcError {
    IpcError::new(
        "offline.unknown-job",
        format!("выгрузки `{job}` нет среди начатых"),
    )
}
