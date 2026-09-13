use tolearn_generate::Step;
use tolearn_generate::ledger::Tally;
use tolearn_generate::plan;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::planned::{PlanOut, RevisePlanIn};
use crate::ipc::planning::{drawn, taken, told};

const KIND: &str = "Переделка карты";

pub fn run(context: &Context, input: &RevisePlanIn) -> Result<PlanOut, IpcError> {
    let previous = taken(&input.plan)?;
    let wish = told("уточнение", &input.wish)?;
    drawn(
        context,
        KIND,
        Step::Revise,
        &input.request,
        &input.level,
        Tally::extend,
        |online, request| plan::revise(online, request, &previous, &wish),
    )
}
