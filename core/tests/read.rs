#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "read gate: a panic here is the report"
)]

use tolearn_core::read::{Kind, blocks};

#[test]
fn пустой_текст_не_даёт_ни_одного_блока() {
    assert!(blocks("").is_empty());
    assert!(blocks("\n\n   \n").is_empty());
}

#[test]
fn соседние_строки_склеиваются_в_один_абзац() {
    let made = blocks("первая строка\nвторая строка\n\nследующий абзац");

    assert_eq!(made.len(), 2);
    assert_eq!(made[0].kind, Kind::Paragraph);
    assert_eq!(made[0].text, "первая строка вторая строка");
    assert_eq!(made[1].text, "следующий абзац");
}

#[test]
fn заголовок_несёт_свой_уровень() {
    let made = blocks("# Верхний\nтекст\n### Третий");

    assert_eq!(made[0].kind, Kind::Heading);
    assert_eq!(made[0].level, 1);
    assert_eq!(made[0].text, "Верхний");
    assert_eq!(made[2].level, 3);
}

#[test]
fn семь_решёток_это_текст_а_не_заголовок() {
    let made = blocks("####### не заголовок");

    assert_eq!(made[0].kind, Kind::Paragraph);
    assert_eq!(made[0].text, "####### не заголовок");
}

#[test]
fn решётка_без_пробела_остаётся_текстом() {
    let made = blocks("#хештег");

    assert_eq!(made[0].kind, Kind::Paragraph);
}

#[test]
fn списки_размечены_по_маркеру_и_по_номеру() {
    let made = blocks("- первый\n* второй\n+ третий\n1. четвёртый");

    assert_eq!(made.len(), 4);
    assert!(made.iter().all(|block| block.kind == Kind::Item));
    assert_eq!(made[3].text, "четвёртый");
}

#[test]
fn цитата_отделяется_от_абзаца() {
    let made = blocks("абзац\n> цитата\nещё абзац");

    assert_eq!(made[0].kind, Kind::Paragraph);
    assert_eq!(made[1].kind, Kind::Quote);
    assert_eq!(made[1].text, "цитата");
    assert_eq!(made[2].kind, Kind::Paragraph);
}

#[test]
fn код_в_заборе_сохраняет_строки_как_есть() {
    let made = blocks("до\n```rust\nlet a = 1;\n\n    let b = 2;\n```\nпосле");

    assert_eq!(made[1].kind, Kind::Code);
    assert_eq!(made[1].text, "let a = 1;\n\n    let b = 2;");
    assert_eq!(made[2].text, "после");
}

#[test]
fn незакрытый_забор_не_теряет_код() {
    let made = blocks("```\nlet a = 1;");

    assert_eq!(made.len(), 1);
    assert_eq!(made[0].kind, Kind::Code);
    assert_eq!(made[0].text, "let a = 1;");
}

#[test]
fn ярлыки_блоков_совпадают_с_протоколом() {
    assert_eq!(Kind::Heading.label(), "heading");
    assert_eq!(Kind::Paragraph.label(), "paragraph");
    assert_eq!(Kind::Item.label(), "item");
    assert_eq!(Kind::Quote.label(), "quote");
    assert_eq!(Kind::Code.label(), "code");
}
