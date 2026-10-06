use std::fmt;

use tolearn_generate::Model;
use tolearn_provider::{CheckError, Provider, Said, Stop, stoppable};

use super::meter::Meter;

pub struct Speaker {
    provider: Provider,
    key: Option<String>,
    pub stop: Stop,
    pub meter: Meter,
}

impl Speaker {
    pub fn new(provider: Provider, key: Option<String>) -> Self {
        Self {
            provider,
            key,
            stop: Stop::default(),
            meter: Meter::default(),
        }
    }
}

impl fmt::Debug for Speaker {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("Speaker")
            .field("active", &self.provider.active)
            .field("meter", &self.meter)
            .finish_non_exhaustive()
    }
}

impl Model for Speaker {
    fn ask(&self, prompt: &str) -> Result<Said, CheckError> {
        let said = stoppable(&self.provider, self.key.as_deref(), prompt, &self.stop);
        self.meter
            .count(said.as_ref().map(|said| said.tokens).unwrap_or_default());
        said
    }
}
