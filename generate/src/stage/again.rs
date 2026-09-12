use tolearn_core::stage::Stage;

use crate::sources::{PAGE_CHARS, excerpt};

use super::gathered::Gathered;
use super::place::Place;
use super::prompt;
use super::{PREVIOUS_CHARS, TEXT_PROMPT_CHARS};

const FRAME: &str = "Этот этап уже написан, но ученик просит объяснить его иначе. Ниже прежний текст этапа: не повторяй его дословно — возьми другой заход к объяснению, другие примеры, другое задание практики и другие вопросы. Источники, правила и формат ответа — те же, что выше.";

const CLOSE: &str = "Ответь снова одним объектом JSON в том же виде.";

pub(super) fn again(place: &Place<'_>, gathered: &Gathered, previous: &Stage) -> String {
    let before = written(previous);
    let tail = format!("\n\n{CLOSE}");
    let wanted = before.chars().count().min(PREVIOUS_CHARS) + tail.chars().count();
    let head = headed(place, gathered, PAGE_CHARS);
    let overflow = (head.chars().count() + wanted).saturating_sub(TEXT_PROMPT_CHARS);
    let head = match overflow {
        0 => head,
        overflow => headed(place, gathered, cap(gathered, overflow)),
    };
    let room = TEXT_PROMPT_CHARS.saturating_sub(head.chars().count() + tail.chars().count());
    let text: String = before.chars().take(room).collect();
    format!("{head}{text}{tail}")
}

fn headed(place: &Place<'_>, gathered: &Gathered, chars: usize) -> String {
    format!(
        "{}\n\n{FRAME}\n\nПрежний текст этапа:\n",
        prompt::text_within(place, gathered, chars)
    )
}

fn cap(gathered: &Gathered, overflow: usize) -> usize {
    let lengths: Vec<usize> = gathered
        .pages
        .iter()
        .map(|visited| excerpt(&visited.page.text).chars().count())
        .collect();
    let within =
        |chars: usize| -> usize { lengths.iter().map(|length| (*length).min(chars)).sum() };
    let limit = within(PAGE_CHARS).saturating_sub(overflow);
    let (mut low, mut high) = (0, PAGE_CHARS);
    while low < high {
        let middle = (low + high).div_ceil(2);
        if within(middle) <= limit {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}

fn written(stage: &Stage) -> String {
    let task = stage.practice.task.iter().map(|block| block.text.clone());
    let questions = stage
        .questions
        .iter()
        .map(|question| format!("- {}", question.text));
    let blocks = stage.blocks.iter().map(|block| block.text.clone());
    let lines: Vec<String> = ["Практика:".to_owned()]
        .into_iter()
        .chain(task)
        .chain([
            format!("Сдаётся: {}", stage.practice.deliverable),
            "Вопросы:".to_owned(),
        ])
        .chain(questions)
        .chain(["Теория:".to_owned()])
        .chain(blocks)
        .collect();
    lines.join("\n\n")
}
