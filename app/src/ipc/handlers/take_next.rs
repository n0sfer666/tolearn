use tolearn_generate::fork;
use tolearn_generate::online;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::forked::{TakeNextIn, TakeNextOut};
use crate::ipc::forking::after;
use crate::ipc::generated::generated;
use crate::ipc::generating::ReadyStage;
use crate::ipc::kitted::kitted;
use crate::ipc::lapsed::lapses;
use crate::ipc::placed::placed;
use crate::ipc::planning::{refused, voiced};
use crate::ipc::running::Finale;

const KIND: &str = "Следующий этап";

pub fn run(context: &Context, input: &TakeNextIn) -> Result<TakeNextOut, IpcError> {
    let after = after(&input.program, &input.node, &input.stage);
    let work = placed(context, "next", &after);
    generated(
        context,
        work,
        |stop| {
            let model = voiced(context, KIND, stop.clone())?;
            let reach = context.reach()?;
            let online = online(reach.as_ref(), &model).map_err(refused)?;
            let choice = usize::try_from(input.choice).unwrap_or(usize::MAX);
            let lapses = lapses(context, &input.program)?;
            let landed = kitted(context, &online, stop, |kit| {
                fork::take(kit, &after, choice, &lapses).map_err(refused)
            })?;
            Ok(TakeNextOut {
                node: landed.node,
                stage: landed.stage,
            })
        },
        |out| {
            Finale::Stage(ReadyStage {
                program: input.program.clone(),
                node: out.node.clone(),
                stage: out.stage.clone(),
            })
        },
    )
}
