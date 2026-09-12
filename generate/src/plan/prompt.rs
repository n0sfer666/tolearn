use tolearn_core::program::MAX_DEPTH;

use super::flaw::{Flaw, span};
use super::shown::shown;
use super::types::{Part, Plan, Request};
use super::{MAX_HOURS, MAX_STAGES, STAGE_MAX_HOURS, STAGE_MIN_HOURS};

const FORMAT: &str = r#"{"title": "название", "slug": "latin-kebab-case", "goal": "что ученик сможет в конце, одной фразой", "volatility": "stable | evolving | volatile", "stages": [{"id": "latin-kebab-case", "title": "название этапа", "hours": [2, 4]}], "children": [{"title": "название подпрограммы", "goal": "цель подпрограммы", "hours": [40, 60]}]}"#;

pub(super) fn task(request: &Request, part: Option<&Part>, depth: usize) -> String {
    let Request {
        request,
        level,
        locale,
    } = request;
    let scope = match part {
        None => "Составь карту всей программы по запросу.".to_owned(),
        Some(part) => format!(
            "Программа уже разбита на подпрограммы. Составь карту одной из них — «{}». Её цель: {} На неё отведено {} ч.",
            part.title,
            part.goal,
            span(part.hours)
        ),
    };
    let split = if depth < MAX_DEPTH {
        format!(
            "Если этапов больше чем на {MAX_HOURS} ч — это узел: только children, у каждой подпрограммы не больше {MAX_HOURS} ч, её этапы не расписывай."
        )
    } else {
        format!(
            "Эта программа на {depth}-м, последнем уровне и не дробится: только stages, не больше {MAX_HOURS} ч."
        )
    };
    format!(
        "Ты составляешь карту учебной программы: цель и строки этапов или подпрограмм с часами, без содержания.\n\
         {scope}\n\n\
         Запрос ученика: {request}\n\
         Уровень ученика: {level}\n\
         Язык программы: {locale} — на нём пиши title, goal и названия строк.\n\n\
         Правила:\n\
         - Этап занимает {STAGE_MIN_HOURS}–{STAGE_MAX_HOURS} ч: теория вместе с практикой. Первый этап — один самый простой инструмент, дальше нагрузка растёт постепенно.\n\
         - Если программа укладывается в {MAX_HOURS} ч — это лист: только stages, не больше {MAX_STAGES} этапов, в сумме не больше {MAX_HOURS} ч.\n\
         - {split}\n\
         - id этапа и slug — строчная латиница, цифры и одиночные дефисы; id не повторяются.\n\
         - hours — [минимум, максимум] в целых часах.\n\
         - volatility: stable — тема меняется десятилетиями, учат по учебникам; evolving — меняется за годы, нужны учебник и документация; volatile — меняется за месяцы, нужны официальная документация и релизы.\n\n\
         Ответь одним объектом JSON без пояснений:\n{FORMAT}"
    )
}

pub(super) fn revised(request: &Request, previous: &Plan, wish: &str) -> String {
    format!(
        "{}\n\n\
         Прежняя карта:\n{}\n\n\
         Уточнение ученика: {wish}\n\
         Перерисуй карту с учётом уточнения: то, чего оно не касается, оставь как было. Ответь снова одним объектом JSON.",
        task(request, None, 1),
        shown(previous)
    )
}

pub(super) fn repair(task: &str, answer: &str, flaws: &[Flaw]) -> String {
    let listed: Vec<String> = flaws.iter().map(|flaw| format!("- {flaw}")).collect();
    format!(
        "{task}\n\n\
         Твой прошлый ответ:\n{answer}\n\n\
         В нём нарушены правила:\n{}\n\n\
         Исправь нарушения и пришли карту целиком, снова одним объектом JSON.",
        listed.join("\n")
    )
}
