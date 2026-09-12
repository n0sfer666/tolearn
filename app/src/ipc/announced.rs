use tolearn_generate::{Progress, Step};

use super::started::GenerationStep;
use super::tools::Herald;

pub struct Announced(Herald);

impl Announced {
    pub fn new(herald: Herald) -> Self {
        Self(herald)
    }

    fn tell(&self, state: &str, step: Step) {
        (self.0)(GenerationStep {
            step: step.label().to_owned(),
            state: state.to_owned(),
            round: round(step),
        });
    }
}

impl Progress for Announced {
    fn began(&self, step: Step) {
        self.tell("began", step);
    }

    fn ended(&self, step: Step) {
        self.tell("ended", step);
    }
}

fn round(step: Step) -> u32 {
    match step {
        Step::Repair(round) => u32::try_from(round).unwrap_or(u32::MAX),
        _ => 0,
    }
}
