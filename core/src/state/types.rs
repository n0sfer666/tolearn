use std::collections::BTreeMap;

use super::choices::{Grade, Pass, Sitting};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub(super) program: String,
    pub workdir: Option<String>,
    pub stages: BTreeMap<String, StageState>,
    pub clarifications: Vec<Clarification>,
}

impl State {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_owned(),
            workdir: None,
            stages: BTreeMap::new(),
            clarifications: Vec::new(),
        }
    }

    pub fn program(&self) -> &str {
        &self.program
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageState {
    pub opened: Option<String>,
    pub passed: Option<Passed>,
    pub ticks: Vec<String>,
    pub answers: BTreeMap<String, String>,
    pub attempts: Vec<Attempt>,
    pub since: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Passed {
    pub on: String,
    pub by: Pass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub on: String,
    pub by: Sitting,
    pub model: Option<String>,
    pub per_question: Vec<Answered>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answered {
    pub id: String,
    pub result: Grade,
    pub missed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clarification {
    pub stage: String,
    pub block: String,
    pub excerpt: String,
    pub turns: Vec<Turn>,
    pub clear: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub asked: Option<String>,
    pub answer: String,
}
