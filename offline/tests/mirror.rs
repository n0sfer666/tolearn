#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::collections::HashMap;

use tolearn_offline::mirror::{Limits, Mirror, Skip, Weight, mirror};
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
        ..Limits::default()
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
fn запрошенная_страница_сохраняется_мимо_llms_txt() {
    let site = Site::new(&[
        (
            "https://docs.test/llms.txt",
            "# Документация\n- [Основы](https://docs.test/basics.md)\n",
        ),
        ("https://docs.test/context", &page(&[], "Длина контекста")),
        ("https://docs.test/basics.md", "# Основы\n"),
    ]);

    let result = mirror("https://docs.test/context", &site, &wide()).unwrap();

    assert!(
        saved(&result).contains(&"https://docs.test/context"),
        "запрошенной страницы нет в зеркале: {:?}",
        saved(&result)
    );
    assert_eq!(
        result.pages[0].url, "https://docs.test/context",
        "запрошенная страница не первая — открывать будут не её"
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
            ..Limits::default()
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

    let result = mirror(
        "https://docs.test/",
        &site,
        &Limits {
            depth: 5,
            pages: 2,
            ..Limits::default()
        },
    )
    .unwrap();

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

#[test]
fn markdown_из_llms_txt_сохраняется_разметкой() {
    let site = Site::new(&[
        (
            "https://docs.test/llms.txt",
            "# Индекс\n- [Контекст](https://docs.test/context.md)\n",
        ),
        ("https://docs.test/", &page(&[], "Заглавная")),
        (
            "https://docs.test/context.md",
            "# Длина контекста\n\nДефолт `4096` токенов.\n\n```yaml\nnum_ctx: 4096\n```\n",
        ),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    let saved = &result
        .pages
        .iter()
        .find(|page| page.url == "https://docs.test/context.md")
        .unwrap()
        .html;
    assert!(
        saved.contains("<h1>Длина контекста</h1>"),
        "заголовок остался сырым markdown: {saved}"
    );
    assert!(saved.contains("<pre><code"), "фенса кода осталась текстом");
    assert!(
        !saved.contains("# Длина контекста"),
        "решётка утекла в текст"
    );
}

#[test]
fn готовый_html_разметкой_не_переписывается() {
    let site = Site::new(&[("https://docs.test/", &page(&[], "Заглавная"))]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert!(result.pages[0].html.contains("<h1>Заглавная</h1>"));
}

#[test]
fn картинки_страницы_скачиваются_и_подменяются_локальными() {
    let site = Site::new(&[
        (
            "https://docs.test/",
            "<html><body><img src=\"/img/scheme.png\"><video poster=\"/img/cover.jpg\" src=\"/v/clip.mp4\"></video></body></html>",
        ),
        ("https://docs.test/img/scheme.png", "PNGBYTES"),
        ("https://docs.test/img/cover.jpg", "JPGBYTES"),
        ("https://docs.test/v/clip.mp4", "MP4BYTES"),
    ]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    let names: Vec<&str> = result
        .assets
        .iter()
        .map(|asset| asset.name.as_str())
        .collect();
    assert_eq!(result.assets.len(), 3, "скачано не всё: {names:?}");
    assert!(names.contains(&"img-scheme.png"), "нет картинки: {names:?}");
    assert!(names.contains(&"v-clip.mp4"), "нет видео: {names:?}");

    let html = &result.pages[0].html;
    assert!(
        !html.contains("https://docs.test/img/scheme.png"),
        "ссылка осталась внешней"
    );
    assert!(
        html.contains("img-scheme.png"),
        "картинка не подменена: {html}"
    );
    assert!(html.contains("v-clip.mp4"), "видео не подменено: {html}");
}

#[test]
fn тяжёлое_вложение_не_тянется() {
    let site = Site::new(&[
        (
            "https://docs.test/",
            "<html><body><img src=\"/big.png\"></body></html>",
        ),
        ("https://docs.test/big.png", "0123456789"),
    ]);

    let limits = Limits {
        depth: 5,
        pages: 50,
        weight: Weight { each: 4, total: 99 },
    };

    let result = mirror("https://docs.test/", &site, &limits).unwrap();

    assert!(
        result.assets.is_empty(),
        "тяжёлая картинка всё равно скачана"
    );
    assert!(
        result.pages[0].html.contains("/big.png"),
        "ссылка на несохранённое вложение потеряна"
    );
}

#[test]
fn недоступное_вложение_не_валит_зеркало() {
    let site = Site::new(&[(
        "https://docs.test/",
        "<html><body><img src=\"/gone.png\"></body></html>",
    )]);

    let result = mirror("https://docs.test/", &site, &wide()).unwrap();

    assert_eq!(saved(&result).len(), 1);
    assert!(result.assets.is_empty());
}
