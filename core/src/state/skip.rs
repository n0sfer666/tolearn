use super::choices::Pass;
use super::key::key;
use super::types::{Passed, State};

impl State {
    pub fn skip(&mut self, node: &str, stage: &str, today: &str) -> bool {
        self.open(node, stage, today);
        let entry = self.stages.entry(key(node, stage)).or_default();
        if entry.passed.is_some() {
            return false;
        }
        entry.passed = Some(Passed {
            on: today.to_owned(),
            by: Pass::Skip,
        });
        true
    }
}
