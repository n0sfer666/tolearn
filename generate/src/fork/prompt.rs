use std::cmp::Ordering;

use crate::plan::{MAX_HOURS, STAGE_MAX_HOURS, STAGE_MIN_HOURS, span};

use super::MAX_ALTERNATIVES;
use super::ahead::Ahead;
use super::flaw::Flaw;

const FORMAT: &str = r#"{"next": {"why": "чем ученику полезен следующий по карте этап", "recommended": true}, "alternatives": [{"id": "latin-kebab-case", "title": "название этапа", "hours": [2, 4], "why": "чем этот этап лучше ведёт к цели", "recommended": false}]}"#;

pub(super) fn task(ahead: &Ahead) -> String {
    let program = &ahead.leaf;
    let passed = program
        .map
        .stages
        .get(ahead.index)
        .map_or("", |row| row.title.as_str());
    let listed: Vec<String> = program
        .map
        .stages
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let mark = match index.cmp(&(ahead.index + 1)) {
                Ordering::Less => "✓",
                Ordering::Equal => "→",
                Ordering::Greater => "·",
            };
            format!(
                "{mark} {}. {} — {}, {} ч",
                index + 1,
                row.id,
                row.title,
                span(row.hours)
            )
        })
        .collect();
    format!(
        "Ты помогаешь ученику выбрать следующий этап учебной программы: он прошёл этап «{passed}» и идёт дальше.\n\n\
         Программа: {}\n\
         Цель: {}\n\
         Уровень ученика: {}\n\
         Язык программы: {} — на нём пиши title и why.\n\n\
         Карта (✓ — пройдено, → — следующий по карте):\n{}\n\n\
         Объясни в next.why, чем ученику полезен следующий по карте этап. Если другие этапы лучше ведут к цели, предложи их в alternatives, не больше {MAX_ALTERNATIVES}: выбранная альтернатива встанет в карту вместо следующего этапа.\n\n\
         Правила:\n\
         - Этап занимает {STAGE_MIN_HOURS}–{STAGE_MAX_HOURS} ч: теория вместе с практикой.\n\
         - id альтернативы — строчная латиница, цифры и одиночные дефисы; не совпадает ни с id в карте, ни с другой альтернативой.\n\
         - С альтернативой вместо следующего этапа карта не больше {MAX_HOURS} ч.\n\
         - hours — [минимум, максимум] в целых часах.\n\
         - recommended — ровно у одного варианта: у next или у одной из альтернатив. Лучше пути по карте нет — пришли пустой alternatives.\n\n\
         Ответь одним объектом JSON без пояснений:\n{FORMAT}",
        program.title,
        program.goal,
        program.level,
        program.generation.locale,
        listed.join("\n")
    )
}

pub(super) fn repair(task: &str, answer: &str, flaws: &[Flaw]) -> String {
    let listed: Vec<String> = flaws.iter().map(|flaw| format!("- {flaw}")).collect();
    format!(
        "{task}\n\n\
         Твой прошлый ответ:\n{answer}\n\n\
         В нём нарушены правила:\n{}\n\n\
         Исправь нарушения и пришли развилку целиком, снова одним объектом JSON.",
        listed.join("\n")
    )
}
