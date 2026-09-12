use std::fmt;

use tolearn_core::Hours;

use crate::plan::{MAX_HOURS, STAGE_MAX_HOURS, STAGE_MIN_HOURS, span};

use super::MAX_ALTERNATIVES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Flaw {
    Unreadable(String),
    Blank(String),
    TooMany(usize),
    StageId(String),
    DuplicateId(String),
    StageHours { id: String, hours: Hours },
    LeafHours { id: String, hours: Hours },
    Recommended(usize),
}

impl fmt::Display for Flaw {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(reason) => write!(
                out,
                "ответ не читается как развилка в JSON по формату: {reason}"
            ),
            Self::Blank(what) => write!(out, "пусто: {what}"),
            Self::TooMany(count) => write!(
                out,
                "альтернатив {count}, а их не больше {MAX_ALTERNATIVES}"
            ),
            Self::StageId(id) => write!(
                out,
                "id этапа «{id}» — не строчная латиница, цифры и одиночные дефисы"
            ),
            Self::DuplicateId(id) => write!(
                out,
                "id этапа «{id}» уже есть в карте или у другой альтернативы"
            ),
            Self::StageHours { id, hours } => write!(
                out,
                "этап «{id}» на {} ч, а этап занимает от {STAGE_MIN_HOURS} до {STAGE_MAX_HOURS} ч и минимум не больше максимума",
                span(*hours)
            ),
            Self::LeafHours { id, hours } => write!(
                out,
                "с этапом «{id}» вместо следующего карта выйдет на {} ч, а лист не больше {MAX_HOURS} ч",
                span(*hours)
            ),
            Self::Recommended(count) => write!(
                out,
                "recommended стоит у {count} вариантов, а нужен ровно у одного"
            ),
        }
    }
}
