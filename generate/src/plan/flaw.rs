use std::fmt;

use tolearn_core::Hours;
use tolearn_core::program::MAX_DEPTH;

use super::{MAX_HOURS, MAX_STAGES, STAGE_MAX_HOURS, STAGE_MIN_HOURS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flaw {
    Unreadable(String),
    Blank(String),
    Slug(String),
    Empty,
    Mixed,
    StageId(String),
    DuplicateId(String),
    StageHours { id: String, hours: Hours },
    LeafHours(Hours),
    LeafStages(usize),
    PartHours { title: String, hours: Hours },
    TooDeep { depth: usize },
}

impl fmt::Display for Flaw {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(reason) => write!(
                out,
                "ответ не читается как карта в JSON по формату: {reason}"
            ),
            Self::Blank(what) => write!(out, "пусто: {what}"),
            Self::Slug(slug) => write!(
                out,
                "slug «{slug}» — не строчная латиница, цифры и одиночные дефисы"
            ),
            Self::Empty => write!(out, "в карте нет ни этапов, ни подпрограмм"),
            Self::Mixed => write!(
                out,
                "в карте и этапы, и подпрограммы, а нужно что-то одно: лист держит этапы, узел — подпрограммы"
            ),
            Self::StageId(id) => write!(
                out,
                "id этапа «{id}» — не строчная латиница, цифры и одиночные дефисы"
            ),
            Self::DuplicateId(id) => write!(out, "id этапа «{id}» повторяется"),
            Self::StageHours { id, hours } => write!(
                out,
                "этап «{id}» на {} ч, а этап занимает от {STAGE_MIN_HOURS} до {STAGE_MAX_HOURS} ч и минимум не больше максимума",
                span(*hours)
            ),
            Self::LeafHours(hours) => write!(
                out,
                "этапы в сумме на {} ч, а лист не больше {MAX_HOURS} ч: раздели карту на подпрограммы",
                span(*hours)
            ),
            Self::LeafStages(count) => write!(
                out,
                "в карте {count} этапов, а лист не больше {MAX_STAGES} этапов"
            ),
            Self::PartHours { title, hours } => write!(
                out,
                "подпрограмма «{title}» на {} ч, а подпрограмма занимает от 1 до {MAX_HOURS} ч и минимум не больше максимума",
                span(*hours)
            ),
            Self::TooDeep { depth } => write!(
                out,
                "программа на уровне {depth} из {MAX_DEPTH} уже не делится на подпрограммы: нужны только этапы"
            ),
        }
    }
}

pub(super) fn span(hours: Hours) -> String {
    format!("{}–{}", hours.min, hours.max)
}
