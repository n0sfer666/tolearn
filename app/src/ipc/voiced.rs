use std::fmt;

use tolearn_generate::Model;
use tolearn_provider::{CheckError, Kind, Provider, Said, Stop, stoppable};

use crate::journal::Journal;

pub struct Voiced {
    provider: Provider,
    key: Option<String>,
    journal: Journal,
    kind: &'static str,
    stop: Stop,
}

impl Voiced {
    pub fn new(
        provider: Provider,
        key: Option<String>,
        journal: Journal,
        kind: &'static str,
        stop: Stop,
    ) -> Self {
        Self {
            provider,
            key,
            journal,
            kind,
            stop,
        }
    }

    pub fn remote(&self) -> bool {
        self.provider.active != Kind::Local
    }

    pub fn named(&self) -> Option<String> {
        let name = match self.provider.active {
            Kind::Local => &self.provider.local.model,
            Kind::Remote => &self.provider.remote.model,
            Kind::Harness => &self.provider.harness.id,
        };
        (!name.is_empty()).then(|| name.clone())
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
        match stoppable(&self.provider, self.key.as_deref(), prompt, &self.stop) {
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
