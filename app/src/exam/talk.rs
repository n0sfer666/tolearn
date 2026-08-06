use std::time::Instant;

use tolearn_provider::{Provider, ask};

use crate::ipc::Context;
use crate::ipc::IpcError;

pub struct Speaker {
    provider: Provider,
    key: Option<String>,
}

impl Speaker {
    pub fn new(context: &Context) -> Result<Self, IpcError> {
        Ok(Self {
            provider: Provider::read(&context.provider()).map_err(crate::ipc::failed)?,
            key: context.vault().key().map_err(crate::ipc::denied)?,
        })
    }

    pub fn ask(&self, prompt: &str) -> Result<Heard, IpcError> {
        let started = Instant::now();
        let said = ask(&self.provider, self.key.as_deref(), prompt).map_err(crate::ipc::refute)?;
        Ok(Heard {
            text: said.text,
            seconds: started.elapsed().as_secs(),
            tokens: said.tokens,
        })
    }
}

pub struct Heard {
    pub text: String,
    pub seconds: u64,
    pub tokens: Option<u32>,
}
