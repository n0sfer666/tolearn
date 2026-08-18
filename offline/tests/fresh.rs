#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use tolearn_offline::fresh::{
    Answer, Freshness, Kept, Mark, WINDOW, body_hash, freshness, weighed,
};

fn sent(html: &str) -> Answer {
    Answer::Sent {
        bytes: html.as_bytes().to_vec(),
        etag: Some("\"v2\"".to_string()),
        last_modified: None,
    }
}

fn kept(hash: &str) -> Kept {
    Kept {
        body_hash: Some(hash.to_string()),
        etag: Some("\"v1\"".to_string()),
        last_modified: Some("Mon, 27 Jul 2026 10:00:00 GMT".to_string()),
    }
}

fn mark(saved: bool, checkable: bool, checked_at: Option<i64>) -> Mark {
    Mark {
        saved,
        checkable,
        checked_at,
    }
}

#[test]
fn голова_страницы_на_хэш_не_влияет() {
    let before =
        "<html><head><meta name='build' content='1'></head><body><h1>Ollama</h1></body></html>";
    let after = "<html><head><meta name='build' content='2'><script src='/a.js?v=9'></script></head><body><h1>Ollama</h1></body></html>";

    assert_eq!(body_hash(before.as_bytes()), body_hash(after.as_bytes()));
}

#[test]
fn правка_внутри_тела_меняет_хэш() {
    let before = "<html><body><h1>Ollama</h1></body></html>";
    let after = "<html><body><h1>Ollama</h1><p>Новое</p></body></html>";

    assert_ne!(body_hash(before.as_bytes()), body_hash(after.as_bytes()));
}

#[test]
fn отступы_и_переносы_на_хэш_не_влияют() {
    let dense = "<body> <h1>Ollama</h1> <p>Текст</p> </body>";
    let loose = "<body>\n  <h1>Ollama</h1>\n\n  <p>Текст</p>\n</body>";

    assert_eq!(body_hash(dense.as_bytes()), body_hash(loose.as_bytes()));
}

#[test]
fn страница_без_тела_хэшируется_целиком() {
    assert_eq!(body_hash(b"plain text"), body_hash(b"plain  text\n"));
    assert_ne!(body_hash(b"plain text"), body_hash(b"other text"));
}

#[test]
fn тело_в_комментарии_за_настоящее_не_принимается() {
    let commented = "<html><head><!-- <body>черновик</body> --><meta name='build' content='1'>\
</head><body><h1>Ollama</h1></body></html>";
    let plain =
        "<html><head><meta name='build' content='2'></head><body><h1>Ollama</h1></body></html>";

    assert_eq!(body_hash(commented.as_bytes()), body_hash(plain.as_bytes()));
}

#[test]
fn строка_body_внутри_скрипта_срез_не_уводит() {
    let scripted = "<html><head><script>var a = \"<body>1</body>\";</script></head>\
<body><h1>Ollama</h1></body></html>";
    let plain = "<html><head></head><body><h1>Ollama</h1></body></html>";

    assert_eq!(body_hash(scripted.as_bytes()), body_hash(plain.as_bytes()));
}

#[test]
fn тег_с_похожим_именем_за_тело_не_считается() {
    let guard = "<html><head><bodyguard>шум</bodyguard></head><body><p>текст</p></body></html>";
    let plain = "<html><head></head><body><p>текст</p></body></html>";

    assert_eq!(body_hash(guard.as_bytes()), body_hash(plain.as_bytes()));
}

#[test]
fn ответ_триста_четыре_перекачки_не_требует() {
    let was = kept("hash-1");

    let change = weighed(&was, &Answer::Same, 100);

    assert!(!change.fetch, "304 потребовал перекачки");
    assert_eq!(change.mark.body_hash.as_deref(), Some("hash-1"));
    assert_eq!(
        change.mark.etag.as_deref(),
        Some("\"v1\""),
        "валидатор стёрт"
    );
    assert_eq!(change.mark.at, 100);
}

#[test]
fn совпавшее_тело_перекачки_не_требует() {
    let html = "<html><head><title>a</title></head><body><p>то же самое</p></body></html>";
    let was = kept(&body_hash(html.as_bytes()));

    let change = weighed(&was, &sent(html), 200);

    assert!(!change.fetch, "неизменное тело потянули заново");
    assert_eq!(
        change.mark.etag.as_deref(),
        Some("\"v2\""),
        "валидатор не обновлён"
    );
    assert_eq!(change.mark.at, 200);
}

#[test]
fn изменившееся_тело_требует_перекачки() {
    let now = "<body><p>новое</p></body>";
    let was = kept(&body_hash("<body><p>старое</p></body>".as_bytes()));

    let change = weighed(&was, &sent(now), 300);

    assert!(change.fetch, "изменение тела не замечено");
    assert_eq!(change.mark.body_hash, Some(body_hash(now.as_bytes())));
}

#[test]
fn слишком_большое_тело_считается_изменившимся() {
    let was = kept("hash-1");

    let change = weighed(
        &was,
        &Answer::Big {
            etag: Some("\"v2\"".to_string()),
            last_modified: None,
        },
        500,
    );

    assert!(change.fetch, "непроверенное тело сочли прежним");
    assert_eq!(
        change.mark.body_hash, None,
        "хэш обрезка сохранён как настоящий"
    );
    assert_eq!(change.mark.etag.as_deref(), Some("\"v2\""));
}

#[test]
fn материал_без_хэша_проверить_нечем_и_он_тянется() {
    let change = weighed(&Kept::default(), &sent("<body>любое</body>"), 400);

    assert!(change.fetch, "материал без хэша признан свежим");
}

#[test]
fn нескачанное_держит_кнопку_на_сохранении() {
    let marks = [mark(true, true, Some(90)), mark(false, true, None)];

    assert_eq!(freshness(&marks, 100, WINDOW), Freshness::Missing);
}

#[test]
fn теме_без_единого_скачиваемого_материала_обновлять_нечего() {
    assert_eq!(freshness(&[], 100, WINDOW), Freshness::Missing);
}

#[test]
fn скачанное_в_окне_свежо_и_помнит_момент() {
    let marks = [mark(true, true, Some(1000)), mark(true, true, Some(900))];

    assert_eq!(
        freshness(&marks, 1000 + WINDOW - 1, WINDOW),
        Freshness::Stale
    );
    assert_eq!(freshness(&marks, 1000, WINDOW), Freshness::Fresh(900));
}

#[test]
fn окно_прошло_и_кнопка_зовёт_обновить() {
    let marks = [mark(true, true, Some(0))];

    assert_eq!(freshness(&marks, WINDOW, WINDOW), Freshness::Stale);
    assert_eq!(freshness(&marks, WINDOW - 1, WINDOW), Freshness::Fresh(0));
}

#[test]
fn скачанное_но_ни_разу_не_проверенное_считается_несвежим() {
    let marks = [mark(true, true, None)];

    assert_eq!(freshness(&marks, 100, WINDOW), Freshness::Stale);
}

#[test]
fn непроверяемое_окна_не_сдвигает() {
    let marks = [mark(true, false, None), mark(true, true, Some(100))];

    assert_eq!(freshness(&marks, 200, WINDOW), Freshness::Fresh(100));
    assert_eq!(
        freshness(&[mark(true, false, None)], 200, WINDOW),
        Freshness::Unchecked,
        "проверять нечего, и время тут ни при чём"
    );
}
