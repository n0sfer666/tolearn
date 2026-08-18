use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{OfflineCostIn, OfflineCostOut};
use crate::ipc::{open, settings};
use crate::offline;

pub fn run(context: &Context, input: &OfflineCostIn) -> Result<OfflineCostOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let budget = settings::stored(context)?.budget_bytes();
    let cost = offline::cost(&context.offline(), budget, &offline::every(&scan));
    Ok(OfflineCostOut {
        materials: count(cost.materials),
        held: count(cost.held),
        used: cost.used,
        budget: cost.budget,
        spare: cost.spare,
        need: cost.need,
        tight: cost.tight,
    })
}

fn count(many: usize) -> u32 {
    u32::try_from(many).unwrap_or(u32::MAX)
}
