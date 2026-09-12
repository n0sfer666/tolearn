use tolearn_core::Hours;
use tolearn_core::program::Volatility;

use crate::sources::{PAGE_CHARS, excerpt};

use super::gathered::{Dropped, Gathered};
use super::place::Place;
use super::{MAX_BOOKS, MAX_IMAGES, MAX_PAGES, MAX_TERMS, MAX_THEORY_CHARS, MIN_THEORY_CHARS};

const PROPOSAL: &str = r#"{"books": [{"title": "название книги", "author": "автор", "isbn": "только если уверен", "chapter": "глава или раздел для этого этапа"}], "pages": [{"url": "https://..."}], "images": [{"query": "запрос к Wikimedia Commons на английском", "caption": "подпись под картинкой"}]}"#;

const STAGE: &str = r#"{"blocks": [{"kind": "heading | paragraph | callout | code | diagram | image", "text": "текст блока", "lang": "язык кода, только у code", "image": "i1, только у image", "sources": ["p1", "b1"]}], "practice": {"task": [{"kind": "paragraph", "text": "условие задания"}], "deliverable": "что ученик сдаёт", "constraints": [{"claim": "ограничение", "check": "команда проверки, если есть", "expect": "что она должна показать"}], "acceptance": [{"claim": "критерий приёмки", "check": "команда проверки, если есть", "expect": "что она должна показать"}]}, "questions": [{"text": "вопрос", "answer": "эталонный ответ"}], "terms": ["термин, который вводит этап"], "tools": ["инструмент, которым пользуется ученик"]}"#;

pub(super) fn sources(place: &Place<'_>) -> String {
    format!(
        "Ты подбираешь источники для одного этапа учебной программы. Текст этапа будет написан по ним позже, а сейчас приложение проверит каждый источник.\n\n\
         {}\n\n\
         Правила:\n\
         - {}\n\
         - Не больше {MAX_BOOKS} книг, {MAX_PAGES} страниц и {MAX_IMAGES} картинок; пустой список лучше выдуманного источника.\n\
         - Книга — существующее издание с автором; chapter — глава или раздел, который относится к этому этапу.\n\
         - Страница — точный адрес статьи или раздела документации, а не главная сайта.\n\
         - Картинка — только если без неё непонятно; query — короткий запрос к Wikimedia Commons на английском.\n\n\
         Ответь одним объектом JSON без пояснений:\n{PROPOSAL}",
        whereabouts(place),
        class(place.program.generation.volatility)
    )
}

pub(super) fn replace(task: &str, answer: &str, dropped: &[Dropped]) -> String {
    let listed: Vec<String> = dropped
        .iter()
        .map(|dropped| format!("- {}: {}", dropped.what, dropped.reason))
        .collect();
    format!(
        "{task}\n\n\
         Твой прошлый ответ:\n{answer}\n\n\
         Приложение не приняло эти источники:\n{}\n\n\
         Предложи замену только для них, принятые не повторяй. Ответь снова одним объектом JSON в том же виде.",
        listed.join("\n")
    )
}

pub(super) fn text(place: &Place<'_>, gathered: &Gathered) -> String {
    text_within(place, gathered, PAGE_CHARS)
}

pub(super) fn text_within(place: &Place<'_>, gathered: &Gathered, chars: usize) -> String {
    format!(
        "Ты пишешь один этап учебной программы: теорию, практику и вопросы с эталонными ответами.\n\n\
         {}\n\
         Язык программы: {} — на нём пиши весь текст.\n\n\
         Проверенные источники:\n{}\n\n\
         Правила:\n{}\n\n\
         Ответь одним объектом JSON без пояснений:\n{STAGE}",
        whereabouts(place),
        place.program.generation.locale,
        listed(gathered, Some(chars)),
        rules(place)
    )
}

pub(super) fn rules(place: &Place<'_>) -> String {
    let tools = if place.first() {
        "Это первый этап программы: в практике ровно один инструмент, самый простой из подходящих; назови его в tools."
    } else {
        "Инструменты, которыми пользуется ученик, назови в tools."
    };
    format!(
        "- Опирайся на источники выше и в sources блока перечисли id тех, на которые он опирается. Других источников нет.\n\
         - Книгу упоминай только названием и главой, без цитат.\n\
         - Ссылка в тексте — только на адрес проверенной страницы.\n\
         - image — только картинка из списка, её id в поле image, подпись в text.\n\
         - diagram — схема на Mermaid в text, без ограды из обратных кавычек.\n\
         - code — код в text, язык в lang.\n\
         - Не пиши id блоков, проверок и вопросов: их проставит приложение.\n\
         - Теория — заголовки, абзацы и врезки — от {MIN_THEORY_CHARS} до {MAX_THEORY_CHARS} знаков; код, схемы и картинки в счёт не идут.\n\
         - Не больше {MAX_TERMS} новых терминов, все они в terms.\n\
         - Практика — одно задание в часы этапа: task — условие блоками, deliverable — что сдаётся, constraints — ограничения, acceptance — критерии приёмки; check — команда, которая проверяет пункт, если она есть.\n\
         - {tools}\n\
         - Вопросы проверяют понимание теории, у каждого эталонный ответ."
    )
}

pub(super) fn whereabouts(place: &Place<'_>) -> String {
    let program = place.program;
    let rows: Vec<String> = program
        .map
        .stages
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let mark = if index == place.index { "→" } else { " " };
            format!(
                "{mark} {}. {} ({}) — {} ч",
                index + 1,
                row.title,
                row.id,
                span(row.hours)
            )
        })
        .collect();
    let order = if place.first() {
        "Это первый этап: ученик ещё ничего не знает."
    } else {
        "Этапы выше ученик уже прошёл — не повторяй их."
    };
    format!(
        "Программа: {}\nЦель: {}\nУровень ученика: {}\n\nКарта, текущий этап отмечен стрелкой:\n{}\n\nЭтап «{}» занимает {} ч вместе с практикой. {order}",
        program.title,
        program.goal,
        program.level,
        rows.join("\n"),
        place.row.title,
        span(place.row.hours)
    )
}

fn class(volatility: Volatility) -> &'static str {
    match volatility {
        Volatility::Stable => {
            "Тема устойчивая (stable): бери учебники и классические книги, страницы — если они подробнее книги."
        }
        Volatility::Evolving => {
            "Тема развивается (evolving): нужны учебник и актуальная документация."
        }
        Volatility::Volatile => {
            "Тема быстро меняется (volatile): бери официальную документацию и заметки о релизах, книги — только свежие."
        }
    }
}

pub(super) fn listed(gathered: &Gathered, excerpts: Option<usize>) -> String {
    let books = gathered.books.iter().enumerate().map(|(index, known)| {
        format!(
            "b{}: книга «{}», {}; глава: {}",
            index + 1,
            known.book.title,
            known.book.authors.join(", "),
            known.chapter
        )
    });
    let pages = gathered.pages.iter().enumerate().map(|(index, known)| {
        let named = format!(
            "p{}: страница «{}», {}",
            index + 1,
            known.page.title.as_deref().unwrap_or(&known.page.url),
            known.page.url
        );
        match excerpts {
            Some(chars) => {
                let cut: String = excerpt(&known.page.text).chars().take(chars).collect();
                format!("{named}\nИзвлечённый текст:\n{cut}")
            }
            None => named,
        }
    });
    let images = gathered
        .images
        .iter()
        .enumerate()
        .map(|(index, image)| format!("i{}: картинка «{}»", index + 1, image.block.text));
    let lines: Vec<String> = books.chain(pages).chain(images).collect();
    if lines.is_empty() {
        "нет — пиши без ссылок на источники".to_owned()
    } else {
        lines.join("\n\n")
    }
}

fn span(hours: Hours) -> String {
    format!("{}–{}", hours.min, hours.max)
}
