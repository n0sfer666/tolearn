use std::time::Instant;

use tolearn_provider::{Provider, ask};

use crate::ipc::Context;
use crate::ipc::IpcError;
use crate::journal::Journal;

const KIND: &str = "Диалог";

pub struct Speaker {
    provider: Provider,
    key: Option<String>,
    journal: Journal,
}

impl Speaker {
    pub fn new(context: &Context) -> Result<Self, IpcError> {
        let provider = Provider::read(&context.provider()).map_err(crate::ipc::failed)?;
        Ok(Self {
            journal: Journal::new(context.llm_log(), provider.journal),
            provider,
            key: context.vault().key().map_err(crate::ipc::denied)?,
        })
    }

    pub fn ask(&self, prompt: &str) -> Result<Heard, IpcError> {
        let started = Instant::now();
        let said = match ask(&self.provider, self.key.as_deref(), prompt) {
            Ok(said) => said,
            Err(error) => {
                let refused = crate::ipc::refute(error);
                self.journal.refused(KIND, prompt, &refused.message);
                return Err(refused);
            }
        };
        self.journal.said(KIND, prompt, &said.text);
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
