pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub enabled: bool,
    pub flavor: Flavor,
    pub endpoint: String,
    pub model: String,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            enabled: false,
            flavor: Flavor::Ollama,
            endpoint: DEFAULT_ENDPOINT.to_owned(),
            model: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    Ollama,
    OpenAi,
}

pub(crate) const FLAVOR: [(&str, Flavor); 2] =
    [("ollama", Flavor::Ollama), ("openai", Flavor::OpenAi)];

impl Flavor {
    pub fn label(self) -> &'static str {
        FLAVOR
            .iter()
            .find(|(_, flavor)| *flavor == self)
            .map_or("ollama", |(label, _)| label)
    }

    pub fn parse(label: &str) -> Option<Self> {
        FLAVOR
            .iter()
            .find(|(known, _)| *known == label)
            .map(|(_, flavor)| *flavor)
    }
}
