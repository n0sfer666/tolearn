use tolearn_core::state::{Attempt, Sitting, State};
use tolearn_generate::exam;
use tolearn_generate::start::day;
use tolearn_generate::{Step, local, online, stepped};

use crate::ipc::clock::now;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::examined::{ExamIn, ExamOut};
use crate::ipc::examining::{answered, paper, staged};
use crate::ipc::planning::{refused, voiced};
use crate::ipc::shelf;

const KIND: &str = "Зачёт";

pub fn run(context: &Context, input: &ExamIn) -> Result<ExamOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let answers = answered(stage, &input.answers)?;
    let claim = context.running().claim()?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let online = if model.remote() {
        let reach = context.reach()?;
        online(reach.as_ref(), &model).map_err(refused)?
    } else {
        local(&model)
    };
    let sheet = paper(&tree.program.uuid, branch.tree, stage, &answers);
    let at = now();
    let progress = context.tools().progress();
    let sat = stepped(progress.as_ref(), Step::Exam, || {
        exam::sit(&online, context.data(), &sheet, at)
    })
    .map_err(refused)?;
    let attempt = Attempt {
        on: day(at),
        by: Sitting::Written,
        model: sat.model.or_else(|| model.named()),
        per_question: sat.per_question,
    };
    let passed = attempt.passes();
    let node = &branch.tree.program.uuid;
    State::update(context.data(), &tree.program.uuid, |state| {
        state.attempt(node, &stage.id, attempt);
    })?;
    Ok(ExamOut { passed })
}
