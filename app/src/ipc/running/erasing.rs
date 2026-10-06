use super::super::generating::GenerationOutcome;
use super::Running;

#[derive(Debug)]
pub struct Erasing {
    running: Running,
    program: String,
    gone: bool,
}

impl Erasing {
    pub(super) fn new(running: Running, program: String) -> Self {
        Self {
            running,
            program,
            gone: false,
        }
    }

    pub fn gone(mut self) {
        self.gone = true;
    }
}

impl Drop for Erasing {
    fn drop(&mut self) {
        self.running.erased(&self.program, self.gone);
    }
}

pub(super) fn owned(outcome: &GenerationOutcome, program: &str) -> bool {
    outcome
        .stage
        .as_ref()
        .map_or(outcome.work.program.as_str(), |stage| {
            stage.program.as_str()
        })
        == program
}
