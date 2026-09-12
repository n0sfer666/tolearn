use tolearn_generate::plan;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::planned::{PlanOut, PlanProgramIn};
use crate::ipc::planning::drawn;

const KIND: &str = "Карта программы";

pub fn run(context: &Context, input: &PlanProgramIn) -> Result<PlanOut, IpcError> {
    drawn(context, KIND, &input.request, &input.level, plan::plan)
}
