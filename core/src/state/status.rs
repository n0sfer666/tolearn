use super::choices::Pass;
use super::key::key;
use super::types::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Fresh,
    Opened,
    Passed(Pass),
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Opened => "opened",
            Self::Passed(_) => "passed",
        }
    }
}

impl State {
    pub fn status(&self, node: &str, stage: &str) -> Status {
        match self.stages.get(&key(node, stage)) {
            None => Status::Fresh,
            Some(entry) => entry
                .passed
                .as_ref()
                .map_or(Status::Opened, |passed| Status::Passed(passed.by)),
        }
    }

    pub fn open(&mut self, node: &str, stage: &str, today: &str) -> bool {
        let entry = self.stages.entry(key(node, stage)).or_default();
        if entry.opened.is_some() {
            return false;
        }
        entry.opened = Some(today.to_owned());
        true
    }
}
