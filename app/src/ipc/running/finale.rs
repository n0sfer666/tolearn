use super::super::error::IpcError;
use super::super::forked::ForkOut;
use super::super::generating::{GenerationOutcome, GenerationWork, ReadyStage, RefusalView};
use super::super::planned::PlanOut;

#[derive(Debug, Clone)]
pub enum Finale {
    Stage(ReadyStage),
    Fork(ForkOut),
    Plan(PlanOut),
    Refused(IpcError),
    Crashed,
}

impl Finale {
    pub(super) fn outcome(self, work: GenerationWork, ended: u64) -> GenerationOutcome {
        let mut outcome = GenerationOutcome {
            work,
            stage: None,
            fork: None,
            plan: None,
            refusal: None,
            seen: false,
            ended,
        };
        match self {
            Self::Stage(stage) => outcome.stage = Some(stage),
            Self::Fork(fork) => outcome.fork = Some(fork),
            Self::Plan(plan) => outcome.plan = Some(plan),
            Self::Refused(error) => outcome.refusal = Some(refusal(error.code, error.message)),
            Self::Crashed => {
                outcome.refusal = Some(refusal(
                    "ipc.crashed".to_owned(),
                    "поток генерации упал: работа оборвана".to_owned(),
                ));
            }
        }
        outcome
    }
}

fn refusal(code: String, message: String) -> RefusalView {
    RefusalView { code, message }
}
