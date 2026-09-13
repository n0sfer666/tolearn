use super::choices::Grade;
use super::types::State;
use crate::program::Tree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lapse {
    pub stage: String,
    pub question: String,
    pub missed: Vec<String>,
}

impl State {
    pub fn lapses(&self, tree: &Tree) -> Vec<Lapse> {
        tree.every_stage()
            .into_iter()
            .filter_map(|(node, id)| {
                let stage = tree.branch(node)?.tree.stages.get(id)?;
                Some((stage, self.last_attempt(node, id)?))
            })
            .flat_map(|(stage, attempt)| {
                attempt
                    .per_question
                    .iter()
                    .filter(|row| row.result != Grade::Ok)
                    .filter_map(move |row| {
                        let question = stage.questions.iter().find(|asked| asked.id == row.id)?;
                        Some(Lapse {
                            stage: stage.title.clone(),
                            question: question.text.clone(),
                            missed: row.missed.clone(),
                        })
                    })
            })
            .collect()
    }
}
