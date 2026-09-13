use std::collections::BTreeMap;

use tolearn_core::state::{Attempt, Sitting, State};
use tolearn_generate::exam::{self, Paper};
use tolearn_generate::start::day;
use tolearn_generate::{Step, local, online, stepped};

use crate::ipc::clock::now;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::examined::{ExamIn, ExamOut};
use crate::ipc::examining::{question, staged};
use crate::ipc::planning::{refused, voiced};
use crate::ipc::shelf;

const KIND: &str = "Зачёт";

pub fn run(context: &Context, input: &ExamIn) -> Result<ExamOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let mut answers = BTreeMap::new();
    for answer in &input.answers {
        let question = question(stage, &answer.id)?;
        if !answer.text.trim().is_empty() {
            answers.insert(question.id.clone(), answer.text.clone());
        }
    }
    if answers.is_empty() {
        return Err(IpcError::new(
            "exam.empty",
            "ответов нет: ответьте хотя бы на один вопрос этапа".to_owned(),
        ));
    }
    let claim = context.running().claim()?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let online = if model.remote() {
        let reach = context.reach()?;
        online(reach.as_ref(), &model).map_err(refused)?
    } else {
        local(&model)
    };
    let node = &branch.tree.program.uuid;
    let program = &branch.tree.program;
    let paper = Paper {
        program: &tree.program.uuid,
        node,
        stage,
        level: &program.level,
        locale: &program.generation.locale,
        answers: &answers,
    };
    let at = now();
    let progress = context.tools().progress();
    let sat = stepped(progress.as_ref(), Step::Exam, || {
        exam::sit(&online, context.data(), &paper, at)
    })
    .map_err(refused)?;
    let attempt = Attempt {
        on: day(at),
        by: Sitting::Written,
        model: sat.model.or_else(|| model.named()),
        per_question: sat.per_question,
    };
    let passed = attempt.passes();
    State::update(context.data(), &tree.program.uuid, |state| {
        state.attempt(node, &stage.id, attempt);
    })?;
    Ok(ExamOut { passed })
}
