use tolearn_core::state::State;
use tolearn_generate::{online, regenerate};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::forking::after;
use crate::ipc::kitted::kitted;
use crate::ipc::planning::{refused, voiced};
use crate::ipc::regenerated::{RegenerateStageIn, RegenerateStageOut};

const KIND: &str = "Перегенерация этапа";

pub fn run(context: &Context, input: &RegenerateStageIn) -> Result<RegenerateStageOut, IpcError> {
    let at = after(&input.program, &input.node, &input.stage);
    let claim = context.running().claim()?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let reach = context.reach()?;
    let online = online(reach.as_ref(), &model).map_err(refused)?;
    kitted(context, &online, claim.stop(), |kit| {
        regenerate::regenerate(kit, &at).map_err(refused)
    })?;
    State::update(context.data(), at.program, |state| {
        state.rewritten(at.node, at.stage);
    })?;
    Ok(RegenerateStageOut {
        stage: input.stage.clone(),
    })
}
