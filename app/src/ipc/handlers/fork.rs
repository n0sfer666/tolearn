use tolearn_generate::{Step, fork, online, stepped};

use crate::ipc::clock::now;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::forked::{ForkIn, ForkOut};
use crate::ipc::forking::{after, view};
use crate::ipc::lapsed::lapses;
use crate::ipc::planning::{refused, voiced};

const KIND: &str = "Развилка";

pub fn run(context: &Context, input: &ForkIn) -> Result<ForkOut, IpcError> {
    let after = after(&input.program, &input.node, &input.stage);
    if let Some(fork) = fork::known(context.data(), &after).map_err(refused)? {
        return Ok(view(&fork));
    }
    let claim = context.running().claim()?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let reach = context.reach()?;
    let online = online(reach.as_ref(), &model).map_err(refused)?;
    let lapses = lapses(context, &input.program)?;
    let progress = context.tools().progress();
    let fork = stepped(progress.as_ref(), Step::Fork, || {
        fork::propose(&online, context.data(), &after, now(), &lapses)
    })
    .map_err(refused)?;
    Ok(view(&fork))
}
