use std::cell::{Cell, RefCell};

use tolearn_generate::Model;
use tolearn_provider::{CheckError, Said, Tokens};

use crate::support::Scripted;

pub const MODEL: &str = "stub-model";

#[derive(Debug)]
pub struct Metered {
    script: Scripted,
    spent: RefCell<Vec<Tokens>>,
    broken: Option<usize>,
    calls: Cell<usize>,
}

impl Metered {
    pub fn new(script: Scripted, mut spent: Vec<Tokens>) -> Self {
        spent.reverse();
        Self {
            script,
            spent: RefCell::new(spent),
            broken: None,
            calls: Cell::new(0),
        }
    }

    pub fn broken(self, at: usize) -> Self {
        Self {
            broken: Some(at),
            ..self
        }
    }
}

impl Model for Metered {
    fn ask(&self, prompt: &str) -> Result<Said, CheckError> {
        let call = self.calls.replace(self.calls.get() + 1);
        if self.broken == Some(call) {
            return Err(CheckError::Unreachable("timeout".to_owned()));
        }
        let mut said = self.script.ask(prompt)?;
        said.tokens = self.spent.borrow_mut().pop().unwrap_or_default();
        said.model = Some(MODEL.to_owned());
        Ok(said)
    }
}

pub fn spent(input: Option<u32>, output: Option<u32>) -> Tokens {
    Tokens { input, output }
}
