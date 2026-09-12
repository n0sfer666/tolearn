#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "export gate: a panic here is the report"
)]

mod support;

use tolearn_core::block::{Block, Kind};
use tolearn_core::export::render;
use tolearn_core::program::{Tree, load};

fn chiptune() -> Tree {
    load(&support::root().join("examples/chiptune")).unwrap()
}

fn page(tree: &Tree, path: &str) -> String {
    render(tree)
        .pages
        .into_iter()
        .find(|page| page.path == path)
        .unwrap()
        .text
}

fn block(tree: &mut Tree, kind: Kind) -> &mut Block {
    tree.stages
        .get_mut("voices")
        .unwrap()
        .blocks
        .iter_mut()
        .find(|block| block.kind == kind)
        .unwrap()
}

fn written(kind: Kind, text: &str) -> String {
    let mut tree = chiptune();
    block(&mut tree, kind).text = text.to_owned();
    page(&tree, "01-voices.md")
}

fn holds(text: &str, expected: &str) {
    assert!(text.contains(expected), "нет `{expected}` в\n{text}");
}

#[test]
fn a_heading_inside_a_paragraph_takes_the_depth_of_the_page() {
    let text = written(Kind::Paragraph, "# Раздел ##\nтекст");

    holds(&text, "\n## Раздел\n\nтекст\n");
    assert!(!text.contains("\n# Раздел"), "{text}");
}

#[test]
fn an_underline_does_not_turn_a_paragraph_into_a_heading() {
    holds(&written(Kind::Paragraph, "текст\n==="), "\nтекст\n\\===\n");
    holds(&written(Kind::Paragraph, "текст\n--"), "\nтекст\n\\--\n");
    holds(&written(Kind::Paragraph, "текст\n---"), "\nтекст\n\n---\n");
}

#[test]
fn a_year_at_the_start_of_a_line_is_not_a_list() {
    holds(
        &written(Kind::Paragraph, "1986. Famicom Disk System"),
        "\n1986\\. Famicom Disk System\n",
    );
    holds(
        &written(Kind::Paragraph, "1. один\n2. два"),
        "\n1. один\n2. два\n",
    );
}

#[test]
fn a_list_after_text_is_set_apart_and_marked_alike() {
    holds(
        &written(Kind::Paragraph, "каналы:\n* импульсный\n+ шумовой"),
        "\nканалы:\n\n- импульсный\n- шумовой\n",
    );
}

#[test]
fn a_fence_inside_a_paragraph_is_closed_and_named() {
    holds(
        &written(Kind::Paragraph, "до\n```c\nint x;"),
        "\nдо\n\n```c\nint x;\n```\n",
    );
    holds(
        &written(Kind::Paragraph, "~~~ py thon\nx = 1\n~~~\nпосле"),
        "\n```text\nx = 1\n```\n\nпосле\n",
    );
}

#[test]
fn a_callout_keeps_its_markup_inside_the_quote() {
    holds(
        &written(Kind::Callout, "Важно:\n- раз\n```\nx\n```"),
        "\n> Важно:\n>\n> - раз\n>\n> ```text\n> x\n> ```\n",
    );
}

#[test]
fn a_heading_ends_neither_in_a_closing_mark_nor_in_a_colon() {
    holds(&written(Kind::Heading, "Канал #"), "\n## Канал \\#\n");
    holds(&written(Kind::Heading, "Итог:"), "\n## Итог\n");
}

#[test]
fn a_language_that_would_break_the_fence_becomes_text() {
    let mut tree = chiptune();
    block(&mut tree, Kind::Code).lang = Some("c`".to_owned());

    holds(&page(&tree, "01-voices.md"), "\n```text\nCPU = 1_789_773\n");
}

#[test]
fn an_asset_with_spaces_and_brackets_keeps_a_working_link() {
    let mut tree = chiptune();
    block(&mut tree, Kind::Image).asset = Some("assets/pulse wave (50).png".to_owned());

    holds(
        &page(&tree, "01-voices.md"),
        "](assets/pulse%20wave%20%2850%29.png)\n",
    );
}

#[test]
fn an_address_inside_a_title_stays_inside_its_link() {
    let mut tree = chiptune();
    let url = tree.program.sources.pages[0].url.clone();
    tree.program.sources.pages[0].title = url.clone();
    tree.program.map.stages[0].title = format!("Этап {url}");
    tree.program.map.stages[1].title = format!("Черновик {url}");

    let text = page(&tree, "index.md");

    holds(&text, &format!("[{url}]({url})"));
    holds(&text, &format!("[Этап {url}](01-voices.md)"));
    holds(&text, &format!("2. Черновик <{url}> — "));
}
