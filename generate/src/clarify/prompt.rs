use tolearn_core::state::Turn;

use crate::latest::latest;
use crate::stage::situated;

use super::doubt::Doubt;
use super::{ANSWER_CHARS, BLOCK_CHARS, CHAIN_CHARS, FRAGMENT_CHARS, QUESTION_CHARS};

const EARLIER: &str =
    "Прежние объяснения этого фрагмента — ученику всё ещё неясно, зайди с другой стороны:";
const PICKED: &str = "Ученик выделил вот это место:";

pub fn prompt(doubt: &Doubt<'_>) -> String {
    let place = &doubt.place;
    format!(
        "Ты объясняешь ученику место в этапе учебной программы, которое ему непонятно.\n\n\
         {}\n\n\
         Этап: «{}».\n\
         Язык программы: {} — на нём пиши объяснение.\n\n\
         {}Непонятный фрагмент этапа:\n{}\n\n\
         {}{}\
         Правила:\n\
         - Объясни фрагмент проще и другими словами, чем в тексте этапа; опирайся на этапы выше по карте и не забегай в следующие.\n\
         - Если ученик выделил место внутри фрагмента, объясняй прежде всего его.\n\
         - Объясняй только этот фрагмент, не пересказывай весь этап.\n\
         - Не длиннее {ANSWER_CHARS} знаков.\n\
         - Разметка: абзацы, списки, врезка — строки, начатые с «> », код — в ограде из трёх обратных кавычек. Без заголовков, картинок, схем и ссылок.\n\n\
         Ответь одним объяснением, без вступления и без вопросов к ученику.",
        situated(place),
        place.row.title,
        place.program.generation.locale,
        picked(doubt.fragment),
        cut(&doubt.block.text, BLOCK_CHARS),
        earlier(doubt.chain),
        asked(doubt.question)
    )
}

fn picked(fragment: Option<&str>) -> String {
    fragment.map_or_else(String::new, |fragment| {
        format!("{PICKED} {}\n\n", cut(fragment, FRAGMENT_CHARS))
    })
}

fn earlier(chain: &[Turn]) -> String {
    let room = CHAIN_CHARS.saturating_sub(EARLIER.chars().count() + 1);
    let kept = latest(chain.iter().map(entry).collect(), room, 2);
    if kept.is_empty() {
        return String::new();
    }
    format!("{EARLIER}\n{}\n\n", kept.join("\n\n"))
}

fn entry(turn: &Turn) -> String {
    match &turn.asked {
        Some(asked) => format!("Вопрос ученика: {asked}\nОбъяснение:\n{}", turn.answer),
        None => format!("Объяснение:\n{}", turn.answer),
    }
}

fn asked(question: Option<&str>) -> String {
    question.map_or_else(String::new, |question| {
        format!("Вопрос ученика: {}\n\n", cut(question, QUESTION_CHARS))
    })
}

fn cut(text: &str, chars: usize) -> String {
    text.chars().take(chars).collect()
}
