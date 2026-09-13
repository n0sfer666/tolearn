use tolearn_generate::fork;
use tolearn_generate::online;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::forked::{TakeNextIn, TakeNextOut};
use crate::ipc::forking::after;
use crate::ipc::kitted::kitted;
use crate::ipc::lapsed::lapses;
use crate::ipc::planning::{refused, voiced};

const KIND: &str = "Следующий этап";

pub fn run(context: &Context, input: &TakeNextIn) -> Result<TakeNextOut, IpcError> {
    let after = after(&input.program, &input.node, &input.stage);
    let claim = context.running().claim()?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let reach = context.reach()?;
    let online = online(reach.as_ref(), &model).map_err(refused)?;
    let choice = usize::try_from(input.choice).unwrap_or(usize::MAX);
    let lapses = lapses(context, &input.program)?;
    let stage = kitted(context, &online, claim.stop(), |kit| {
        fork::take(kit, &after, choice, &lapses).map_err(refused)
    })?;
    Ok(TakeNextOut { stage })
}
