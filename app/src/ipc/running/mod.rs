mod claim;
mod erasing;
mod finale;
mod hold;
mod marked;

pub use claim::Claim;
pub use erasing::Erasing;
pub use finale::Finale;
pub use hold::Hold;
pub use marked::Marked;

use std::collections::BTreeSet;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use tolearn_provider::Stop;

use super::clock::millis;
use super::error::IpcError;
use super::generating::{GenerationOutcome, GenerationStateOut, StepMark};
use super::started::GenerationStep;

pub type Crier = Arc<dyn Fn(GenerationStateOut) + Send + Sync>;

#[derive(Debug)]
struct Held {
    stop: Stop,
    hold: Hold,
}

#[derive(Debug, Default)]
struct State {
    held: Option<Held>,
    outcome: Option<GenerationOutcome>,
    erased: BTreeSet<String>,
}

#[derive(Clone, Default)]
pub struct Running {
    state: Arc<Mutex<State>>,
    voice: Arc<Mutex<()>>,
    crier: Arc<OnceLock<Crier>>,
}

impl Running {
    pub fn herald(&self, crier: Crier) {
        let _ = self.crier.set(crier);
    }

    pub fn claim(&self, mut hold: Hold) -> Result<Claim, IpcError> {
        let stop = Stop::default();
        let held = stop.clone();
        self.spoken(move |state| {
            if let Some(held) = &state.held {
                return (Err(held.hold.busy()), false);
            }
            if hold
                .program()
                .is_some_and(|uuid| state.erased.contains(uuid))
            {
                return (
                    Err(IpcError::new(
                        "generate.erased",
                        "эту программу сейчас удаляют: генерация не начата".to_owned(),
                    )),
                    false,
                );
            }
            let generation = match hold.work_mut() {
                Some(work) => {
                    work.began = millis();
                    work.marks.clear();
                    state.outcome = None;
                    true
                }
                None => false,
            };
            state.held = Some(Held { stop: held, hold });
            (Ok(()), generation)
        })?;
        Ok(Claim::new(self.clone(), stop))
    }

    pub fn erasing(&self, program: &str) -> Result<Erasing, IpcError> {
        let mut state = self.state();
        if state
            .held
            .as_ref()
            .is_some_and(|held| held.hold.program() == Some(program))
        {
            return Err(IpcError::new(
                "delete.busy",
                "идёт генерация этой программы: дождитесь её или отмените".to_owned(),
            ));
        }
        state.erased.insert(program.to_owned());
        Ok(Erasing::new(self.clone(), program.to_owned()))
    }

    pub fn cancel(&self) -> bool {
        self.state()
            .held
            .as_ref()
            .is_some_and(|held| held.stop.stop())
    }

    pub fn view(&self) -> GenerationStateOut {
        view(&self.state())
    }

    pub fn seen(&self) -> GenerationStateOut {
        self.spoken(|state| {
            let had = state.outcome.take();
            let changed = had
                .as_ref()
                .is_some_and(|outcome| !outcome.seen || outcome.plan.is_none());
            state.outcome = had.and_then(|mut outcome| {
                outcome.seen = true;
                outcome.plan.is_some().then_some(outcome)
            });
            (view(state), changed)
        })
    }

    pub(super) fn mark(&self, step: GenerationStep) {
        let mut state = self.state();
        if let Some(work) = state.held.as_mut().and_then(|held| held.hold.work_mut()) {
            work.marks.push(StepMark { step, at: millis() });
        }
    }

    fn release(&self, finale: Option<Finale>) {
        self.spoken(|state| {
            let Some(Held {
                hold: Hold::Generation(work),
                ..
            }) = state.held.take()
            else {
                return ((), false);
            };
            state.outcome = finale.map(|finale| finale.outcome(*work, millis()));
            ((), true)
        });
    }

    fn erased(&self, program: &str, gone: bool) {
        self.spoken(|state| {
            state.erased.remove(program);
            let stale = gone
                && state
                    .outcome
                    .as_ref()
                    .is_some_and(|outcome| erasing::owned(outcome, program));
            if stale {
                state.outcome = None;
            }
            ((), stale)
        });
    }

    fn spoken<T>(&self, change: impl FnOnce(&mut State) -> (T, bool)) -> T {
        let _voice = self.voice.lock().unwrap_or_else(PoisonError::into_inner);
        let (out, told) = {
            let mut state = self.state();
            let (out, loud) = change(&mut state);
            (out, loud.then(|| view(&state)))
        };
        if let (Some(crier), Some(told)) = (self.crier.get(), told) {
            crier(told);
        }
        out
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for Running {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("Running")
            .field("state", &self.state)
            .field("heralded", &self.crier.get().is_some())
            .finish()
    }
}

fn view(state: &State) -> GenerationStateOut {
    GenerationStateOut {
        work: state
            .held
            .as_ref()
            .and_then(|held| held.hold.work())
            .cloned(),
        outcome: state.outcome.clone(),
    }
}
