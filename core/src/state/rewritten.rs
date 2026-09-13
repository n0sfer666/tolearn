use super::key::key;
use super::types::State;

impl State {
    pub fn rewritten(&mut self, node: &str, stage: &str) {
        if let Some(entry) = self.stages.get_mut(&key(node, stage)) {
            entry.since = entry.attempts.len();
            entry.ticks.clear();
            entry.answers.clear();
        }
    }
}
