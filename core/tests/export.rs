#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "export gate: a panic here is the report"
)]

mod support;

use tolearn_core::export::{Export, render};
use tolearn_core::program::{Tree, load};

const CHIPTUNE: &str = "examples/chiptune";
const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
const TOOLS: &str = "b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56";
const ROM: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

fn tree(relative: &str) -> Tree {
    load(&support::root().join(relative)).unwrap()
}

fn page<'a>(export: &'a Export, path: &str) -> &'a str {
    export
        .pages
        .iter()
        .find(|page| page.path == path)
        .map(|page| page.text.as_str())
        .unwrap_or_else(|| panic!("нет страницы {path}"))
}

fn paths(export: &Export) -> Vec<&str> {
    export.pages.iter().map(|page| page.path.as_str()).collect()
}

fn line<'a>(text: &'a str, start: &str) -> &'a str {
    text.lines()
        .find(|line| line.starts_with(start))
        .unwrap_or_else(|| panic!("нет строки `{start}` в\n{text}"))
}

#[test]
fn a_program_exports_an_index_and_a_page_per_generated_stage() {
    let export = render(&tree(CHIPTUNE));

    assert_eq!(paths(&export), ["index.md", "01-voices.md"]);
    let assets: Vec<(&str, &str)> = export
        .assets
        .iter()
        .map(|asset| (asset.from.as_str(), asset.to.as_str()))
        .collect();
    assert_eq!(
        assets,
        [
            ("assets/pulse-wave.png", "assets/pulse-wave.png"),
            ("assets/voices.svg", "assets/voices.svg"),
        ]
    );
}

#[test]
fn the_index_links_generated_stages_and_marks_the_rest() {
    let export = render(&tree(CHIPTUNE));
    let index = page(&export, "index.md");

    assert!(index.starts_with("# Chiptune: музыка звукового чипа NES\n"));
    assert!(line(index, "- Цель: ").contains("Написать и проиграть"));
    assert_eq!(line(index, "- Объём по карте"), "- Объём по карте: 7–11 ч");
    assert_eq!(line(index, "1. "), "1. [Голоса чипа](01-voices.md) — 2–3 ч");
    let pending = line(index, "2. ");
    assert!(!pending.contains("]("), "{pending}");
    assert!(pending.ends_with(" — 2–4 ч, ждёт генерации"), "{pending}");
}

#[test]
fn the_index_names_the_sources_with_their_check_date() {
    let export = render(&tree(CHIPTUNE));
    let index = page(&export, "index.md");

    assert!(index.contains("\n## Источники\n"), "{index}");
    let book = line(index, "- Karen Collins — Game Sound:");
    assert!(book.contains(", 2. Push Start Button"), "{book}");
    assert!(book.contains(", ISBN 9780262033787"), "{book}");
    assert!(book.ends_with(", проверено 2026-09-11"), "{book}");
    assert_eq!(
        line(index, "- [NESdev Wiki: APU]"),
        "- [NESdev Wiki: APU](https://www.nesdev.org/wiki/APU), проверено 2026-09-11"
    );
}

#[test]
fn subprograms_export_into_numbered_folders_with_a_way_back() {
    let export = render(&tree(NES_DEV));

    assert_eq!(
        paths(&export),
        [
            "index.md",
            "01-build-tools/index.md",
            "01-build-tools/01-first-rom/index.md",
            "01-build-tools/01-first-rom/01-first-rom.md",
            "01-build-tools/01-first-rom/02-linker.md",
        ]
    );
    let root = page(&export, "index.md");
    assert!(line(root, "1. ").contains("[Инструменты сборки](01-build-tools/index.md)"));
    let sound = line(root, "2. ");
    assert!(!sound.contains("]("), "{sound}");
    assert!(sound.ends_with("ждёт генерации"), "{sound}");
    assert!(!root.contains("../index.md"), "{root}");
    let tools = page(&export, "01-build-tools/index.md");
    assert!(
        tools.contains("\n← [Разработка игр для NES](../index.md)\n"),
        "{tools}"
    );
    let pixel = export
        .assets
        .iter()
        .find(|asset| asset.to == "01-build-tools/01-first-rom/assets/pixel.png")
        .unwrap();
    assert_eq!(
        pixel.from,
        format!("children/{TOOLS}/children/{ROM}/assets/pixel.png")
    );
}

#[test]
fn every_page_is_a_tidy_markdown_document() {
    for relative in [CHIPTUNE, NES_DEV] {
        for page in render(&tree(relative)).pages {
            let text = &page.text;
            assert!(text.starts_with("# "), "{}", page.path);
            assert!(
                text.ends_with('\n') && !text.ends_with("\n\n"),
                "{}",
                page.path
            );
            assert!(
                !text.contains("\n\n\n"),
                "{}: лишние пустые строки",
                page.path
            );
            assert!(
                !text.lines().any(|line| line.ends_with(' ')),
                "{}: пробелы в конце строки",
                page.path
            );
        }
    }
}

#[test]
fn labels_follow_the_language_of_the_program() {
    let mut english = tree(CHIPTUNE);
    english.program.generation.locale = "en".to_owned();

    let export = render(&english);

    let index = page(&export, "index.md");
    assert!(index.contains("\n## Stages\n"), "{index}");
    assert!(
        line(index, "2. ").ends_with(", not generated yet"),
        "{index}"
    );
    assert_eq!(line(index, "- Map estimate"), "- Map estimate: 7–11 h");
    assert!(!index.contains("Этапы"), "{index}");
    assert!(page(&export, "01-voices.md").contains("\n## Practice\n"));
}
