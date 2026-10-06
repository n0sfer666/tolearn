#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "export gate: a panic here is the report"
)]

mod support;

use tolearn_core::block::{Block, Kind};
use tolearn_core::export::render;
use tolearn_core::program::{Tree, load};
use tolearn_core::stage::Stage;

fn chiptune() -> Tree {
    load(&support::root().join("examples/chiptune")).unwrap()
}

fn voices(tree: &Tree) -> String {
    render(tree)
        .pages
        .into_iter()
        .find(|page| page.path == "01-voices.md")
        .unwrap()
        .text
}

fn stage(tree: &mut Tree) -> &mut Stage {
    tree.stages.get_mut("voices").unwrap()
}

fn block(stage: &mut Stage, kind: Kind) -> &mut Block {
    stage
        .blocks
        .iter_mut()
        .find(|block| block.kind == kind)
        .unwrap()
}

fn said(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[test]
fn the_page_opens_with_the_stage_and_a_way_back() {
    let text = voices(&chiptune());

    assert!(
        text.starts_with("# Голоса чипа\n\n← [Chiptune: музыка звукового чипа NES](index.md)\n"),
        "{text}"
    );
}

#[test]
fn every_block_kind_becomes_markdown() {
    let text = voices(&chiptune());

    for expected in [
        "\n## Пять каналов 2A03\n",
        "Два **импульсных** канала",
        "\n![Схема](assets/voices.svg)\n",
        "\n![Импульсная волна со скважностью 50 %: половину периода сигнал наверху, половину внизу.](assets/pulse-wave.png)\n",
        "\nЛицензия: CC0-1.0 · Автор: toLearn contributors\n",
        "\n```python\nCPU = 1_789_773\n\ndef timer(hz: float) -> int:\n",
        "\n> Таймер меньше 8 глушит импульсный канал",
        "[APU в NESdev Wiki](https://www.nesdev.org/wiki/APU)",
    ] {
        assert!(text.contains(expected), "нет `{expected}` в\n{text}");
    }
}

#[test]
fn practice_leaves_whole() {
    let tree = chiptune();
    let text = voices(&tree);
    let practice = &tree.stages["voices"].practice;

    assert!(text.contains("\n## Практика\n"), "{text}");
    for block in &practice.task {
        assert!(text.contains(&said(&block.text)), "{}", block.id);
    }
    assert!(text.contains(&format!("- Результат: {}", said(&practice.deliverable))));
    assert!(text.contains("\n### Ограничения\n") && text.contains("\n### Приёмка\n"));
    for check in practice.constraints.iter().chain(&practice.acceptance) {
        assert!(text.contains(&said(&check.claim)), "{}", check.id);
        assert!(text.contains(&said(&check.expect)), "{}", check.id);
    }
}

#[test]
fn questions_leave_without_their_answers() {
    let tree = chiptune();
    let text = voices(&tree);

    for question in &tree.stages["voices"].questions {
        assert!(text.contains(&said(&question.text)), "{}", question.id);
        assert!(
            !text.contains(&said(&question.answer)),
            "{} утёк ответ",
            question.id
        );
    }
    assert!(text.ends_with("\n\nЭталонные ответы не экспортированы: этап не зачтён.\n"));
}

#[test]
fn an_image_source_is_named_under_the_picture() {
    let mut tree = chiptune();
    block(stage(&mut tree), Kind::Image).source = Some("https://example.org/wave".to_owned());

    let text = voices(&tree);

    assert!(
        text.contains("Автор: toLearn contributors · Источник: <https://example.org/wave>\n"),
        "{text}"
    );
}

#[test]
fn brackets_in_an_image_caption_do_not_break_the_picture() {
    let mut tree = chiptune();
    block(stage(&mut tree), Kind::Image).text = "Волна [50 %]".to_owned();

    let text = voices(&tree);

    assert!(
        text.contains("\n![Волна \\[50 %\\]](assets/pulse-wave.png)\n"),
        "{text}"
    );
}

#[test]
fn a_fence_inside_code_gets_a_longer_fence() {
    let mut tree = chiptune();
    block(stage(&mut tree), Kind::Code).text = "```\nx\n```".to_owned();

    let text = voices(&tree);

    assert!(text.contains("\n````python\n```\nx\n```\n````\n"), "{text}");
}

#[test]
fn a_bare_address_becomes_an_autolink_but_code_stays_code() {
    let mut tree = chiptune();
    block(stage(&mut tree), Kind::Paragraph).text =
        "Смотрите https://example.org/apu. И `curl http://localhost:1234` руками.".to_owned();

    let text = voices(&tree);

    assert!(
        text.contains("Смотрите <https://example.org/apu>."),
        "{text}"
    );
    assert!(text.contains("`curl http://localhost:1234`"), "{text}");
    assert!(!text.contains("<http://localhost:1234>"), "{text}");
}

#[test]
fn a_paragraph_keeps_its_lines_and_loses_blank_runs() {
    let mut tree = chiptune();
    block(stage(&mut tree), Kind::Paragraph).text = "- один\n- два\n\n\n\nтри".to_owned();
    block(stage(&mut tree), Kind::Callout).text = "раз\n\n\nдва".to_owned();

    let text = voices(&tree);

    assert!(text.contains("\n- один\n- два\n\nтри\n"), "{text}");
    assert!(text.contains("\n> раз\n>\n> два\n"), "{text}");
}

#[test]
fn a_heading_inside_the_practice_sits_below_the_practice() {
    let mut tree = chiptune();
    stage(&mut tree).practice.task.insert(
        0,
        Block {
            id: "0000beef".to_owned(),
            kind: Kind::Heading,
            text: "Шаги".to_owned(),
            lang: None,
            asset: None,
            license: None,
            attribution: None,
            source: None,
        },
    );

    let text = voices(&tree);

    assert!(text.contains("\n## Практика\n\n### Шаги\n"), "{text}");
}
