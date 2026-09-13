use super::choices::{Grade, Pass};
use super::key::key;
use super::types::{Attempt, Passed, State};

impl Attempt {
    pub fn passes(&self) -> bool {
        !self.per_question.is_empty() && self.per_question.iter().all(|row| row.result == Grade::Ok)
    }
}

impl State {
    pub fn attempt(&mut self, node: &str, stage: &str, attempt: Attempt) {
        self.open(node, stage, &attempt.on);
        let entry = self.stages.entry(key(node, stage)).or_default();
        let examined = entry
            .passed
            .as_ref()
            .is_some_and(|passed| passed.by == Pass::Exam);
        if attempt.passes() && !examined {
            entry.passed = Some(Passed {
                on: attempt.on.clone(),
                by: Pass::Exam,
            });
        }
        entry.attempts.push(attempt);
    }

    pub fn last_attempt(&self, node: &str, stage: &str) -> Option<&Attempt> {
        let entry = self.stages.get(&key(node, stage))?;
        entry.attempts.get(entry.since..)?.last()
    }
}
