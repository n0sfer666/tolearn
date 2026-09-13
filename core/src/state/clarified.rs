use super::key::key;
use super::types::{Clarification, State, Turn};

impl State {
    pub fn clarifications_of(&self, node: &str, stage: &str) -> Vec<(usize, &Clarification)> {
        let at = key(node, stage);
        self.clarifications
            .iter()
            .enumerate()
            .filter(|(_, chain)| chain.stage == at)
            .collect()
    }

    pub fn chain(&self, node: &str, stage: &str, index: usize) -> Option<&Clarification> {
        let at = key(node, stage);
        self.clarifications
            .get(index)
            .filter(|chain| chain.stage == at)
    }

    pub fn chain_mut(
        &mut self,
        node: &str,
        stage: &str,
        index: usize,
    ) -> Option<&mut Clarification> {
        let at = key(node, stage);
        self.clarifications
            .get_mut(index)
            .filter(|chain| chain.stage == at)
    }

    pub fn clarify(
        &mut self,
        node: &str,
        stage: &str,
        block: &str,
        excerpt: &str,
        turn: Turn,
    ) -> usize {
        self.clarifications.push(Clarification {
            stage: key(node, stage),
            block: block.to_owned(),
            excerpt: excerpt.to_owned(),
            turns: vec![turn],
            clear: false,
        });
        self.clarifications.len() - 1
    }

    pub fn unclarify(&mut self, node: &str, stage: &str, index: usize) -> bool {
        if self.chain(node, stage, index).is_none() {
            return false;
        }
        self.clarifications.remove(index);
        true
    }
}
