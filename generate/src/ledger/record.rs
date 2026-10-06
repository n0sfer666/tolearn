use std::fmt;

use serde::{Deserialize, Serialize};
use tolearn_provider::Said;

use crate::step::Step;

use super::kind::Kind;

const NO_DATA: &str = "нет данных";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub step: String,
    pub round: Option<usize>,
    pub kind: Kind,
    pub target: Option<String>,
    pub program: Option<String>,
    pub stage: Option<String>,
    pub at: Option<i64>,
    pub ms: u64,
    pub model: Option<String>,
    pub input: Option<u32>,
    pub output: Option<u32>,
    pub ok: bool,
}

impl Record {
    pub(crate) fn asked(step: Step, round: Option<usize>, ms: u64, said: Option<&Said>) -> Self {
        Self {
            step: step.label().to_owned(),
            round,
            kind: Kind::Model,
            target: None,
            program: None,
            stage: None,
            at: None,
            ms,
            model: said.and_then(|said| said.model.clone()),
            input: said.and_then(|said| said.tokens.input),
            output: said.and_then(|said| said.tokens.output),
            ok: said.is_some(),
        }
    }

    pub(crate) fn checked(kind: Kind, target: &str, ms: u64, ok: bool) -> Self {
        Self {
            step: Step::Sources.label().to_owned(),
            round: None,
            kind,
            target: Some(target.to_owned()),
            program: None,
            stage: None,
            at: None,
            ms,
            model: None,
            input: None,
            output: None,
            ok,
        }
    }
}

impl fmt::Display for Record {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}", self.step)?;
        if let Some(round) = self.round {
            write!(out, " {round}")?;
        }
        if self.kind == Kind::Model {
            write!(
                out,
                " · {} мс · {} · вход {} · выход {}",
                self.ms,
                known(self.model.as_deref()),
                known(self.input),
                known(self.output)
            )?;
        } else {
            write!(
                out,
                " · {} {} · {} мс",
                self.kind.label(),
                self.target.as_deref().unwrap_or_default(),
                self.ms
            )?;
        }
        if !self.ok {
            write!(out, " · отказ")?;
        }
        Ok(())
    }
}

fn known(value: Option<impl fmt::Display>) -> String {
    value.map_or_else(|| NO_DATA.to_owned(), |value| value.to_string())
}
