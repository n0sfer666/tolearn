use tolearn_core::program::Tree;
use tolearn_core::stage::{Question, Stage};

use super::error::IpcError;

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
