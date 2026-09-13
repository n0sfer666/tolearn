use std::collections::BTreeMap;

use tolearn_core::program::Tree;
use tolearn_core::stage::{Question, Stage};
use tolearn_generate::exam::Paper;

use super::error::IpcError;
use super::examined::AnswerView;

pub fn staged<'a>(tree: &'a Tree, stage: &str) -> Result<&'a Stage, IpcError> {
    tree.stages.get(stage).ok_or_else(|| {
        IpcError::new(
            "stage.absent",
            format!("этапа `{stage}` нет или он ещё не сгенерирован"),
        )
    })
}

pub fn question<'a>(stage: &'a Stage, id: &str) -> Result<&'a Question, IpcError> {
    stage
        .questions
        .iter()
        .find(|question| question.id == id)
        .ok_or_else(|| {
            IpcError::new(
                "question.absent",
                format!("у этапа `{}` нет вопроса `{id}`", stage.id),
            )
        })
}

pub fn answered(
    stage: &Stage,
    answers: &[AnswerView],
) -> Result<BTreeMap<String, String>, IpcError> {
    let mut given = BTreeMap::new();
    for answer in answers {
        let question = question(stage, &answer.id)?;
        if !answer.text.trim().is_empty() {
            given.insert(question.id.clone(), answer.text.clone());
        }
    }
    if given.is_empty() {
        return Err(IpcError::new(
            "exam.empty",
            "ответов нет: ответьте хотя бы на один вопрос этапа".to_owned(),
        ));
    }
    Ok(given)
}

pub fn paper<'a>(
    program: &'a str,
    tree: &'a Tree,
    stage: &'a Stage,
    answers: &'a BTreeMap<String, String>,
) -> Paper<'a> {
    Paper {
        program,
        node: &tree.program.uuid,
        stage,
        level: &tree.program.level,
        locale: &tree.program.generation.locale,
        answers,
    }
}
