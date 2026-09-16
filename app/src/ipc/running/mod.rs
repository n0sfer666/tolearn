mod claim;
mod erasing;

pub use claim::Claim;
pub use erasing::Erasing;

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tolearn_provider::Stop;

use super::error::IpcError;

#[derive(Debug, Clone)]
struct Held {
    stop: Stop,
    program: Option<String>,
}

#[derive(Debug, Default)]
struct State {
    held: Option<Held>,
    erased: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Running(Arc<Mutex<State>>);

impl Running {
    pub fn claim(&self, program: Option<&str>) -> Result<Claim, IpcError> {
        let mut state = self.state();
        if state.held.is_some() {
            return Err(IpcError::new(
                "generate.busy",
                "уже идёт генерация: дождитесь её или отмените".to_owned(),
            ));
        }
        if program.is_some_and(|uuid| state.erased.contains(uuid)) {
            return Err(IpcError::new(
                "generate.erased",
                "эту программу сейчас удаляют: генерация не начата".to_owned(),
            ));
        }
        let stop = Stop::default();
        state.held = Some(Held {
            stop: stop.clone(),
            program: program.map(str::to_owned),
        });
        Ok(Claim::new(self.clone(), stop))
    }

    pub fn erasing(&self, program: &str) -> Result<Erasing, IpcError> {
        let mut state = self.state();
        if generating(&state, program) {
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

    fn release(&self) {
        self.state().held = None;
    }

    fn erased(&self, program: &str) {
        self.state().erased.remove(program);
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn generating(state: &State, program: &str) -> bool {
    state
        .held
        .as_ref()
        .is_some_and(|held| held.program.as_deref() == Some(program))
}
