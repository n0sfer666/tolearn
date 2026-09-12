mod claim;

pub use claim::Claim;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tolearn_provider::Stop;

use super::error::IpcError;

#[derive(Debug, Clone, Default)]
pub struct Running(Arc<Mutex<Option<Stop>>>);

impl Running {
    pub fn claim(&self) -> Result<Claim, IpcError> {
        let mut current = self.held();
        if current.is_some() {
            return Err(IpcError::new(
                "generate.busy",
                "уже идёт генерация: дождитесь её или отмените".to_owned(),
            ));
        }
        let stop = Stop::default();
        *current = Some(stop.clone());
        Ok(Claim::new(self.clone(), stop))
    }

    pub fn cancel(&self) -> bool {
        self.held().as_ref().is_some_and(Stop::stop)
    }

    fn release(&self) {
        *self.held() = None;
    }

    fn held(&self) -> MutexGuard<'_, Option<Stop>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
