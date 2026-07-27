use tolearn_core::registry::{Listed, Registry};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{Card, ProgramsIn, ProgramsOut, Tally};

pub fn run(context: &Context, input: &ProgramsIn) -> Result<ProgramsOut, IpcError> {
    let registry = Registry::read(&context.registry())?;
    Ok(ProgramsOut {
        programs: registry
            .entries()
            .iter()
            .map(|listed| card(context, listed, &input.today))
            .collect(),
    })
}

fn card(context: &Context, listed: &Listed, today: &str) -> Card {
    let path = listed.program.path.display().to_string();
    Card {
        id: listed.program.id.clone(),
        title: listed.program.title.clone(),
        path: path.clone(),
        reachable: listed.reachable,
        opened_at: listed.program.opened_at.clone(),
        tally: listed
            .reachable
            .then(|| tally(context, &path, today))
            .flatten(),
    }
}

fn tally(context: &Context, path: &str, today: &str) -> Option<Tally> {
    super::program::run(
        context,
        &crate::ipc::types::ProgramIn {
            bundle: path.to_owned(),
            today: today.to_owned(),
        },
    )
    .ok()
    .map(|program| program.program)
}
