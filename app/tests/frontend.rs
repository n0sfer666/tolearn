#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

#[test]
fn страницы_интерфейса_встроены_в_бинарник() {
    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();

    for page in ["index.html", "ru/index.html", "en/index.html"] {
        assert!(
            context.assets().get(&page.into()).is_some(),
            "{page} отсутствует в бинарнике: окно откроется пустым"
        );
    }
}
