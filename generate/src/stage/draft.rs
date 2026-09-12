use tolearn_core::stage::Stage;

use super::flaw::Flaw;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub stage: Stage,
    pub terms: Vec<String>,
    pub tools: Vec<String>,
    pub cited: Vec<String>,
    pub flaws: Vec<Flaw>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drafted {
    pub said: String,
    pub draft: Result<Draft, Flaw>,
}
