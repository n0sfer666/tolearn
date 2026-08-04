use crate::preset::CLAUDE;

pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434";
pub const OPENAI_ENDPOINT: &str = "http://127.0.0.1:8080/v1";
pub const DEFAULT_TIMEOUT_SECS: u32 = 180;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub enabled: bool,
    pub active: Kind,
    pub local: Http,
    pub remote: Http,
    pub harness: Harness,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Kind::Local,
            local: Http::local(),
            remote: Http::remote(),
            harness: CLAUDE.harness(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Local,
    Remote,
    Harness,
}

pub(crate) const KIND: [(&str, Kind); 3] = [
    ("local", Kind::Local),
    ("remote", Kind::Remote),
    ("harness", Kind::Harness),
];

impl Kind {
    pub fn label(self) -> &'static str {
        KIND.iter()
            .find(|(_, kind)| *kind == self)
            .map_or("local", |(label, _)| label)
    }

    pub fn parse(label: &str) -> Option<Self> {
        KIND.iter()
            .find(|(known, _)| *known == label)
            .map(|(_, kind)| *kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Api {
    Ollama,
    OpenAi,
}

pub(crate) const API: [(&str, Api); 2] = [("ollama", Api::Ollama), ("openai", Api::OpenAi)];

impl Api {
    pub fn label(self) -> &'static str {
        API.iter()
            .find(|(_, api)| *api == self)
            .map_or("ollama", |(label, _)| label)
    }

    pub fn parse(label: &str) -> Option<Self> {
        API.iter()
            .find(|(known, _)| *known == label)
            .map(|(_, api)| *api)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Http {
    pub endpoint: String,
    pub api: Api,
    pub model: String,
}

impl Http {
    pub fn local() -> Self {
        Self {
            endpoint: DEFAULT_ENDPOINT.to_owned(),
            api: Api::Ollama,
            model: String::new(),
        }
    }

    pub fn remote() -> Self {
        Self {
            endpoint: String::new(),
            api: Api::OpenAi,
            model: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Harness {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub timeout_secs: u32,
}
