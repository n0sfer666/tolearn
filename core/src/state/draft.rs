use std::collections::BTreeMap;

use super::key::key;
use super::types::State;

impl State {
    pub fn drafts(&self, node: &str, stage: &str) -> BTreeMap<String, String> {
        self.stages
            .get(&key(node, stage))
            .map(|entry| entry.answers.clone())
            .unwrap_or_default()
    }

    pub fn draft(&mut self, node: &str, stage: &str, question: &str, text: &str) {
        if text.trim().is_empty() {
            if let Some(entry) = self.stages.get_mut(&key(node, stage)) {
                entry.answers.remove(question);
            }
            return;
        }
        self.stages
            .entry(key(node, stage))
            .or_default()
            .answers
            .insert(question.to_owned(), text.to_owned());
    }
}
