#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use tolearn_offline::reader::read;

const LAYOUTS: [&str; 5] = [
    "semantic.html",
    "divs.html",
    "docs.html",
    "table.html",
    "nested.html",
];

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("../fixtures/valid/reader/{name}")).unwrap()
}

#[test]
fn основной_текст_вынут_из_пяти_вёрсток() {
    for name in LAYOUTS {
        let reading = read(&fixture(name), "https://a.test/x");

        assert!(reading.extracted, "{name}: читалка сдалась");
        assert!(
            reading
                .text
                .contains("у каждого значения есть ровно один владелец"),
            "{name}: основной текст потерян"
        );
    }
}

#[test]
fn шум_вокруг_текста_отброшен() {
    for name in LAYOUTS {
        let reading = read(&fixture(name), "https://a.test/x");

        assert!(
            !reading.text.contains("Купите подписку"),
            "{name}: реклама осталась в статье"
        );
        assert!(
            !reading.text.contains("Глава 2"),
            "{name}: навигация осталась в статье"
        );
    }
}

#[test]
fn заголовок_статьи_вынут() {
    let reading = read(&fixture("semantic.html"), "https://a.test/x");

    let title = reading.title.unwrap();

    assert!(
        title.starts_with("Владение"),
        "заголовок статьи потерян: {title}"
    );
}

#[test]
fn при_неудаче_отдаётся_исходный_архив() {
    let source = fixture("menu-only.html");

    let reading = read(&source, "https://a.test/x");

    assert!(!reading.extracted, "оглавление принято за статью");
    assert_eq!(reading.html, source, "вместо архива отдана пустота");
}

#[test]
fn мусор_вместо_html_не_роняет_читалку() {
    let source = "\u{0}не html вовсе";

    let reading = read(source, "https://a.test/x");

    assert!(!reading.extracted);
    assert_eq!(reading.html, source);
}

#[test]
fn у_вынутой_статьи_остаётся_разметка() {
    let reading = read(&fixture("divs.html"), "https://a.test/x");

    assert!(reading.html.contains("<p"), "разметка абзацев потеряна");
    assert!(
        !reading.html.contains("Купите подписку"),
        "боковая колонка попала в статью"
    );
    assert!(!reading.html.contains("<nav"), "навигация попала в статью");
}
