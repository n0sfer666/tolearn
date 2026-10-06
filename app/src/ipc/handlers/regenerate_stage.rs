use tolearn_core::state::State;
use tolearn_generate::{online, regenerate};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::forking::after;
use crate::ipc::generated::generated;
use crate::ipc::generating::ReadyStage;
use crate::ipc::kitted::kitted;
use crate::ipc::placed::placed;
use crate::ipc::planning::{refused, voiced};
use crate::ipc::regenerated::{RegenerateStageIn, RegenerateStageOut};
use crate::ipc::running::Finale;

const KIND: &str = "Перегенерация этапа";

pub fn run(context: &Context, input: &RegenerateStageIn) -> Result<RegenerateStageOut, IpcError> {
    let at = after(&input.program, &input.node, &input.stage);
    let work = placed(context, "regenerate", &at);
    generated(
        context,
        work,
        |stop| {
            let model = voiced(context, KIND, stop.clone())?;
            let reach = context.reach()?;
            let online = online(reach.as_ref(), &model).map_err(refused)?;
            kitted(context, &online, stop, |kit| {
                regenerate::regenerate(kit, &at).map_err(refused)
            })?;
            State::update(context.data(), at.program, |state| {
                state.rewritten(at.node, at.stage);
            })?;
            Ok(RegenerateStageOut {
                stage: input.stage.clone(),
            })
        },
        |out| {
            Finale::Stage(ReadyStage {
                program: input.program.clone(),
                node: at.node.to_owned(),
                stage: out.stage.clone(),
            })
        },
    )
}
