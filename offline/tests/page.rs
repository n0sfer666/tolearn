#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use tolearn_offline::page::{AsFetched, Fetching, PageError, Prerenderer, Source, save};
use url::Url;

struct Disk;

impl Source for Disk {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        let path = Url::parse(url)
            .ok()
            .and_then(|address| address.to_file_path().ok())
            .ok_or_else(|| {
                PageError::Unreachable(url.to_string(), "адрес не ведёт к файлу".to_string())
            })?;
        std::fs::read(path)
            .map_err(|error| PageError::Unreachable(url.to_string(), error.to_string()))
    }
}

struct Missing;

impl Source for Missing {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        Err(PageError::Unreachable(
            url.to_string(),
            "нет сети".to_string(),
        ))
    }
}

struct Marking;

impl Prerenderer for Marking {
    fn render(&self, _url: &str, html: &[u8]) -> Result<Vec<u8>, PageError> {
        let mut rendered = String::from_utf8(html.to_vec()).unwrap();
        rendered = rendered.replace("</body>", "<p id=\"prerendered\">после JS</p></body>");
        Ok(rendered.into_bytes())
    }
}

fn fixture() -> String {
    let path = std::fs::canonicalize("../fixtures/valid/page/index.html").unwrap();
    Url::from_file_path(path).unwrap().to_string()
}

fn sealed() -> Fetching {
    Fetching {
        domains: Some(Vec::new()),
        ..Fetching::default()
    }
}

fn saved() -> String {
    let page = save(&fixture(), &Disk, &AsFetched, &sealed()).unwrap();
    String::from_utf8(page.html).unwrap()
}

#[test]
fn ресурсы_встроены_и_наружу_не_смотрят() {
    let html = saved();

    for attribute in ["src=\"http", "src=\"//", "href=\"http"] {
        let external = html.match_indices(attribute).filter(|(at, _)| {
            let head = html[..*at].rfind('<').unwrap_or(0);
            !html[head..*at].starts_with("<a ")
        });
        assert_eq!(
            external.count(),
            0,
            "в архиве остался внешний ресурс: {attribute}"
        );
    }
    assert!(html.contains("data:image/png"), "картинка не встроена");
    assert!(html.contains("data:text/css"), "стили не встроены");
}

#[test]
fn скрипты_в_архив_не_попадают() {
    let html = saved();

    assert!(!html.contains("__tracker"), "встроенный скрипт сохранён");
    assert!(!html.contains("__loaded"), "внешний скрипт сохранён");
}

#[test]
fn документ_изолирован_от_сети() {
    let html = saved();

    let policy = html.to_lowercase();

    assert!(
        policy.contains("content-security-policy"),
        "нет политики безопасности"
    );
    assert!(
        policy.contains("default-src"),
        "политика не запрещает запросы наружу"
    );
}

#[test]
fn недоступный_ресурс_не_валит_сохранение() {
    let html = saved();

    assert!(html.contains("Каждое значение"), "текст страницы потерян");
    assert!(
        html.contains("баннер"),
        "узел недоступной картинки выброшен"
    );
}

#[test]
fn ссылки_на_страницы_остаются_кликабельными() {
    let html = saved();

    assert!(
        html.contains("https://example.test/next"),
        "ссылка на соседнюю страницу переписана в никуда"
    );
}

#[test]
fn заглушка_пререндера_возвращает_исходный_html() {
    let source = "<html><body>без изменений</body></html>".as_bytes();

    let rendered = AsFetched.render("https://a.test/x", source).unwrap();

    assert_eq!(rendered, source);
}

#[test]
fn пререндер_влияет_на_архив() {
    let page = save(&fixture(), &Disk, &Marking, &sealed()).unwrap();
    let html = String::from_utf8(page.html).unwrap();

    assert!(html.contains("после JS"), "результат пререндера потерян");
}

#[test]
fn заголовок_страницы_вынут() {
    let page = save(&fixture(), &Disk, &AsFetched, &sealed()).unwrap();

    assert_eq!(page.title.as_deref(), Some("Владение в Rust"));
}

#[test]
fn недоступная_страница_доезжает_ошибкой() {
    let failure = save("https://a.test/x", &Missing, &AsFetched, &sealed()).unwrap_err();

    assert!(
        matches!(failure, PageError::Unreachable(url, _) if url == "https://a.test/x"),
        "ошибка загрузки потеряла адрес"
    );
}
