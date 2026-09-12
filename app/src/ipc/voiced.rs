use std::fmt;

use tolearn_generate::Model;
use tolearn_provider::{CheckError, Provider, Said, ask};

use crate::journal::Journal;

pub struct Voiced {
    provider: Provider,
    key: Option<String>,
    journal: Journal,
    kind: &'static str,
}

impl Voiced {
    pub fn new(
        provider: Provider,
        key: Option<String>,
        journal: Journal,
        kind: &'static str,
    ) -> Self {
        Self {
            provider,
            key,
            journal,
            kind,
        }
    }
}

impl fmt::Debug for Voiced {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("Voiced")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl Model for Voiced {
    fn ask(&self, prompt: &str) -> Result<Said, CheckError> {
        match ask(&self.provider, self.key.as_deref(), prompt) {
            Ok(said) => {
                self.journal.said(self.kind, prompt, &said.text);
                Ok(said)
            }
            Err(error) => {
                self.journal.refused(self.kind, prompt, &error.to_string());
                Err(error)
            }
        }
    }
}
