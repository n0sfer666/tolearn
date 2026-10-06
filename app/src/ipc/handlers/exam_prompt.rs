use tolearn_generate::exam;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::examined::{ExamIn, ExamPromptOut};
use crate::ipc::examining::{answered, paper, staged};
use crate::ipc::shelf;

pub fn run(context: &Context, input: &ExamIn) -> Result<ExamPromptOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let answers = answered(stage, &input.answers)?;
    let prompt = exam::prompt(&paper(&tree.program.uuid, branch.tree, stage, &answers));
    Ok(ExamPromptOut { prompt })
}
