use super::choices::Pass;
use super::status::Status;
use super::types::State;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Summary {
    pub passed: u32,
    pub total: u32,
    pub skipped: u32,
}

impl State {
    pub fn summary<'a>(&self, stages: impl IntoIterator<Item = (&'a str, &'a str)>) -> Summary {
        let mut summary = Summary::default();
        for (node, stage) in stages {
            summary.total = summary.total.saturating_add(1);
            if let Status::Passed(by) = self.status(node, stage) {
                summary.passed = summary.passed.saturating_add(1);
                if by == Pass::Skip {
                    summary.skipped = summary.skipped.saturating_add(1);
                }
            }
        }
        summary
    }
}
