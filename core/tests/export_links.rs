#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "export links gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeSet;

use tolearn_core::block::Kind;
use tolearn_core::export::{Export, render};
use tolearn_core::program::{Tree, load};

fn targets(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(place) = rest.find("](") {
        rest = &rest[place + 2..];
        let (target, after) = match rest.strip_prefix('<') {
            Some(inner) => inner.split_once('>').unwrap(),
            None => rest.split_once(')').unwrap(),
        };
        assert!(
            rest.starts_with('<') || !target.contains(char::is_whitespace),
            "`{target}` рвёт ссылку"
        );
        found.push(target.to_owned());
        rest = after;
    }
    found
}

fn autolinks(text: &str) -> Vec<&str> {
    text.match_indices("<http")
        .filter(|(place, _)| !text[..*place].ends_with("]("))
        .map(|(place, _)| {
            let rest = &text[place + 1..];
            &rest[..rest.find('>').unwrap()]
        })
        .collect()
}

fn decoded(target: &str) -> String {
    let mut bytes = Vec::new();
    let mut place = 0;
    while place < target.len() {
        if target[place..].starts_with('%') {
            bytes.push(u8::from_str_radix(&target[place + 1..place + 3], 16).unwrap());
            place += 3;
        } else {
            bytes.push(target.as_bytes()[place]);
            place += 1;
        }
    }
    String::from_utf8(bytes).unwrap()
}

fn resolved(page: &str, target: &str) -> String {
    let mut parts: Vec<&str> = page.split('/').collect();
    parts.pop();
    for piece in target.split('/') {
        match piece {
            ".." => {
                parts.pop();
            }
            "." => {}
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn checked(export: &Export) -> usize {
    let files: BTreeSet<&str> = export
        .pages
        .iter()
        .map(|page| page.path.as_str())
        .chain(export.assets.iter().map(|asset| asset.to.as_str()))
        .collect();
    let mut checked = 0;
    for page in &export.pages {
        for link in autolinks(&page.text) {
            assert!(
                !link.contains(char::is_whitespace),
                "{}: `{link}`",
                page.path
            );
        }
        for target in targets(&page.text) {
            if target.starts_with("http") {
                continue;
            }
            let found = resolved(&page.path, &decoded(&target));
            assert!(
                files.contains(found.as_str()),
                "{}: `{target}` ведёт в невыгруженный `{found}`",
                page.path
            );
            checked += 1;
        }
    }
    checked
}

fn chiptune() -> Tree {
    load(&support::root().join("examples/chiptune")).unwrap()
}

#[test]
fn every_relative_link_leads_to_an_exported_file() {
    for relative in ["examples/chiptune", "fixtures/v2/valid/nes-dev"] {
        let export = render(&load(&support::root().join(relative)).unwrap());

        assert!(checked(&export) > 3, "{relative}: ссылок почти нет");
    }
}

#[test]
fn hostile_titles_and_names_keep_every_link_working() {
    let mut tree = chiptune();
    let asset = "assets/pulse wave (50).png".to_owned();
    tree.assets.remove("assets/pulse-wave.png");
    tree.assets.insert(asset.clone());
    let image = tree.stages.get_mut("voices").unwrap();
    let image = image
        .blocks
        .iter_mut()
        .find(|block| block.kind == Kind::Image)
        .unwrap();
    image.asset = Some(asset);
    let source = &mut tree.program.sources.pages[0];
    source.url = "https://example.org/wiki/APU (2A03)".to_owned();
    source.title = source.url.clone();
    tree.program.map.stages[0].title = "Этап https://example.org/a".to_owned();
    tree.program.map.stages[1].title = "Черновик https://example.org/b.".to_owned();

    let export = render(&tree);

    assert!(checked(&export) > 3);
    let index = &export.pages[0].text;
    assert!(
        index.contains("(<https://example.org/wiki/APU (2A03)>)"),
        "{index}"
    );
}
