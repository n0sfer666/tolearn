use tolearn_generate::Step;
use tolearn_generate::ledger::Tally;
use tolearn_generate::plan;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::planned::{PlanOut, PlanProgramIn};
use crate::ipc::planning::{asked, drawn};

const KIND: &str = "Карта программы";

pub fn run(context: &Context, input: &PlanProgramIn) -> Result<PlanOut, IpcError> {
    let request = asked(&input.request, &input.level, &input.locale)?;
    drawn(
        context,
        KIND,
        Step::Plan,
        &request,
        Tally::replace,
        plan::plan,
    )
}
