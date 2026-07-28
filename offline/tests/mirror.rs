#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::collections::HashMap;

use tolearn_offline::mirror::{Limits, Mirror, Skip, mirror};
use tolearn_offline::page::{PageError, Source};

struct Site {
    pages: HashMap<String, String>,
}

impl Site {
    fn new(pages: &[(&str, &str)]) -> Self {
        Self {
            pages: pages
                .iter()
                .map(|(url, html)| ((*url).to_string(), (*html).to_string()))
                .collect(),
        }
    }
}

impl Source for Site {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        match self.pages.get(url) {
            Some(html) => Ok(html.as_bytes().to_vec()),
            None => Err(PageError::Unreachable(url.to_string(), "404".to_string())),
        }
    }
}

fn page(links: &[&str], text: &str) -> String {
    let body: String = links
        .iter()
        .map(|href| format!("<a href=\"{href}\">дальше</a>"))
        .collect();
    format!("<!doctype html><html><body><h1>{text}</h1>{body}</body></html>")
}

fn saved(result: &Mirror) -> Vec<&str> {
    result.pages.iter().map(|page| page.url.as_str()).collect()
}

fn wide() -> Limits {
    Limits {
        depth: 5,
        pages: 50,
    }
}

#[test]
fn llms_txt_предпочитается_обходу() {
    let site = Site::new(&[
        (
            "https://docs.test/llms.txt",
            "# Документация\n- [Основы](https://docs.test/basics)\n- [Практика](https://docs.test/practice)\n",
        ),
        ("https://docs.test/", &page(&["/tour"], "Заглавная")),
        ("https://docs.test/basics", &page(&["/tour"], "Основы")),
        ("https://docs.test/practice", &page(&[], "Практика")),
        ("https://docs.test/tour", &page(&[], "Экскурсия")),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert!(saved(&result).contains(&"https://docs.test/basics"));
    assert!(saved(&result).contains(&"https://docs.test/practice"));
    assert!(
        !saved(&result).contains(&"https://docs.test/tour"),
        "обход пошёл мимо llms.txt"
    );
}

#[test]
fn без_llms_txt_идёт_обход() {
    let site = Site::new(&[
        ("https://docs.test/", &page(&["/basics"], "Заглавная")),
        ("https://docs.test/basics", &page(&[], "Основы")),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert_eq!(saved(&result).len(), 2);
    assert!(saved(&result).contains(&"https://docs.test/basics"));
}

#[test]
fn robots_txt_соблюдается() {
    let site = Site::new(&[
        (
            "https://docs.test/robots.txt",
            "User-agent: *\nDisallow: /private/\nAllow: /private/open\n",
        ),
        (
            "https://docs.test/",
            &page(&["/private/secret", "/private/open"], "Заглавная"),
        ),
        ("https://docs.test/private/secret", &page(&[], "Секрет")),
        ("https://docs.test/private/open", &page(&[], "Открытое")),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert!(
        !saved(&result).contains(&"https://docs.test/private/secret"),
        "запрещённая страница скачана"
    );
    assert!(
        saved(&result).contains(&"https://docs.test/private/open"),
        "разрешающее правило проигнорировано"
    );
    assert!(
        result
            .skipped
            .iter()
            .any(|(url, why)| url == "https://docs.test/private/secret" && *why == Skip::Robots),
        "причина пропуска не названа"
    );
}

#[test]
fn краулер_держится_домена() {
    let site = Site::new(&[
        (
            "https://docs.test/",
            &page(&["https://other.test/page", "/basics"], "Заглавная"),
        ),
        ("https://docs.test/basics", &page(&[], "Основы")),
        ("https://other.test/page", &page(&[], "Чужое")),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert!(!saved(&result).contains(&"https://other.test/page"));
    assert!(
        result
            .skipped
            .iter()
            .any(|(url, why)| url == "https://other.test/page" && *why == Skip::Domain)
    );
}

#[test]
fn глубина_ограничена() {
    let site = Site::new(&[
        ("https://docs.test/", &page(&["/one"], "Заглавная")),
        ("https://docs.test/one", &page(&["/two"], "Первая")),
        ("https://docs.test/two", &page(&[], "Вторая")),
    ]);

    let result = mirror(
        "https://docs.test/",
        &site,
        &Limits {
            depth: 1,
            pages: 50,
        },
    )
    .unwrap();

    assert_eq!(saved(&result).len(), 2);
    assert!(!saved(&result).contains(&"https://docs.test/two"));
    assert!(
        result
            .skipped
            .iter()
            .any(|(url, why)| url == "https://docs.test/two" && *why == Skip::Depth)
    );
}

#[test]
fn потолок_числа_страниц_соблюдается() {
    let site = Site::new(&[
        (
            "https://docs.test/",
            &page(&["/one", "/two", "/three"], "Заглавная"),
        ),
        ("https://docs.test/one", &page(&[], "Первая")),
        ("https://docs.test/two", &page(&[], "Вторая")),
        ("https://docs.test/three", &page(&[], "Третья")),
    ]);

    let result = mirror("https://docs.test/", &site, &Limits { depth: 5, pages: 2 }).unwrap();

    assert_eq!(saved(&result).len(), 2);
    assert!(result.skipped.iter().any(|(_, why)| *why == Skip::Cap));
}

#[test]
fn ссылки_между_сохранёнными_страницами_локальные() {
    let site = Site::new(&[
        (
            "https://docs.test/",
            &page(&["/basics", "https://other.test/page"], "Заглавная"),
        ),
        ("https://docs.test/basics", &page(&[], "Основы")),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();
    let root = result
        .pages
        .iter()
        .find(|page| page.url == "https://docs.test/")
        .unwrap();
    let basics = result
        .pages
        .iter()
        .find(|page| page.url == "https://docs.test/basics")
        .unwrap();

    assert!(
        root.html.contains(&format!("href=\"{}\"", basics.name)),
        "ссылка на соседнюю страницу не переписана: {}",
        root.html
    );
    assert!(
        root.html.contains("https://other.test/page"),
        "ссылка наружу потеряна"
    );
}

#[test]
fn недоступная_страница_попадает_в_пропуски() {
    let site = Site::new(&[("https://docs.test/", &page(&["/gone"], "Заглавная"))]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert_eq!(saved(&result), vec!["https://docs.test/"]);
    assert!(
        result
            .skipped
            .iter()
            .any(|(url, why)| url == "https://docs.test/gone" && *why == Skip::Unreachable)
    );
}

#[test]
fn недоступный_корень_это_ошибка() {
    let site = Site::new(&[]);

    let failure = mirror("https://docs.test/", &site, &wide()).unwrap_err();

    assert!(matches!(failure, PageError::Unreachable(url, _) if url == "https://docs.test/"));
}

#[test]
fn цикл_ссылок_не_зацикливает() {
    let site = Site::new(&[
        ("https://docs.test/", &page(&["/loop"], "Заглавная")),
        ("https://docs.test/loop", &page(&["/"], "Кольцо")),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert_eq!(saved(&result).len(), 2, "страница сохранена дважды");
}
