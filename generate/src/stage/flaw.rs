use std::fmt;

use super::part::Part;
use super::{FIRST_STAGE_TOOLS, MAX_TERMS, MAX_THEORY_CHARS, MIN_THEORY_CHARS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flaw {
    Unreadable(String),
    UnknownImage { block: String, image: String },
    UnknownSource { block: String, source: String },
    ForeignLink { block: String, url: String },
    BlankBlock { block: String },
    Format { block: String, reason: String },
    NoTheory,
    TheoryLength(usize),
    Terms(usize),
    Tools(Vec<String>),
    Practice(String),
    Questions(String),
}

impl Flaw {
    pub(super) fn part(&self) -> Option<Part> {
        match self {
            Self::Unreadable(_) => None,
            Self::UnknownImage { block, .. }
            | Self::UnknownSource { block, .. }
            | Self::ForeignLink { block, .. }
            | Self::BlankBlock { block }
            | Self::Format { block, .. } => Some(Part::Block(block.clone())),
            Self::NoTheory | Self::TheoryLength(_) | Self::Terms(_) => Some(Part::Theory),
            Self::Tools(_) | Self::Practice(_) => Some(Part::Practice),
            Self::Questions(_) => Some(Part::Questions),
        }
    }
}

impl fmt::Display for Flaw {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(reason) => write!(out, "ответ не прочитан: {reason}"),
            Self::UnknownImage { block, image } => write!(
                out,
                "блок {block}: картинки «{image}» нет среди проверенных"
            ),
            Self::UnknownSource { block, source } => write!(
                out,
                "блок {block}: источника «{source}» нет среди проверенных"
            ),
            Self::ForeignLink { block, url } => {
                write!(
                    out,
                    "блок {block}: ссылка {url} ведёт не на проверенную страницу"
                )
            }
            Self::BlankBlock { block } => write!(out, "блок {block}: пустой текст"),
            Self::Format { block, reason } => write!(out, "блок {block}: {reason}"),
            Self::NoTheory => write!(out, "в теории нет ни одного абзаца или врезки"),
            Self::TheoryLength(chars) => write!(
                out,
                "текст теории — {chars} знаков, а нужно от {MIN_THEORY_CHARS} до {MAX_THEORY_CHARS}: считаются заголовки, абзацы и врезки"
            ),
            Self::Terms(count) => write!(
                out,
                "этап вводит {count} новых терминов, а можно не больше {MAX_TERMS}"
            ),
            Self::Tools(tools) => write!(
                out,
                "в первом этапе программы ровно {FIRST_STAGE_TOOLS} инструмент, а в tools: {}",
                if tools.is_empty() {
                    "пусто".to_owned()
                } else {
                    tools.join(", ")
                }
            ),
            Self::Practice(reason) => write!(out, "практика: {reason}"),
            Self::Questions(reason) => write!(out, "вопросы: {reason}"),
        }
    }
}
