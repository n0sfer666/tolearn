use super::types::{StageState, State};

impl State {
    pub fn active(&self) -> Option<String> {
        self.stages
            .values()
            .flat_map(dates)
            .max()
            .map(str::to_owned)
    }
}

fn dates(stage: &StageState) -> impl Iterator<Item = &str> {
    stage
        .opened
        .as_deref()
        .into_iter()
        .chain(stage.passed.as_ref().map(|passed| passed.on.as_str()))
        .chain(stage.attempts.iter().map(|attempt| attempt.on.as_str()))
}
