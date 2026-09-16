use tolearn_core::program::position;
use tolearn_generate::online;
use tolearn_generate::start;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::kitted::kitted;
use crate::ipc::planning::{asked, refused, taken, told, voiced};
use crate::ipc::started::{StartProgramIn, StartProgramOut};

const KIND: &str = "Этап программы";

pub fn run(context: &Context, input: &StartProgramIn) -> Result<StartProgramOut, IpcError> {
    told("уровень", &input.level)?;
    let request = asked(&input.request, &input.level, &input.locale)?;
    let plan = taken(&input.plan)?;
    let claim = context.running().claim(None)?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let reach = context.reach()?;
    let online = online(reach.as_ref(), &model).map_err(refused)?;
    let program = kitted(context, &online, claim.stop(), |kit| {
        online.tally().extend(context.book().take());
        start::start(kit, &request, &plan).map_err(|error| {
            context.ledger().restore(online.tally().release());
            refused(error)
        })
    })?;
    Ok(opened(context, program))
}

fn opened(context: &Context, program: String) -> StartProgramOut {
    let tree = context.library().open(&program).ok();
    let at = tree.as_ref().and_then(position);
    StartProgramOut {
        node: at
            .map(|at| at.node.program.uuid.clone())
            .unwrap_or_default(),
        stage: at.map(|at| at.row.id.clone()).unwrap_or_default(),
        program,
    }
}
