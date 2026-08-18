#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "manifest gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use serde_json::Value;

const PAGES: [(&str, [&str; 2]); 2] = [
    ("docs/ru/release.md", ["12 МБ", "14 МБ"]),
    ("docs/en/release.md", ["12 MB", "14 MB"]),
];
const OVERLAY: &str = "app/tauri.with-speech.conf.json";
const PACKAGE: &str = ".github/workflows/package.yml";
const RELEASE: &str = ".github/workflows/release.yml";

fn root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name)
}

fn read(name: &str) -> String {
    let path = root(name);
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(reason) => panic!("{}: {reason}", path.display()),
    }
}

fn overlay() -> Value {
    serde_json::from_str(&read(OVERLAY)).unwrap()
}

#[test]
fn вариант_отличается_одним_флагом() {
    let manifest = read("app/Cargo.toml");

    assert!(
        manifest.contains("speech = [\"tolearn-speech/speech\"]"),
        "у `app` нет фичи `speech`: варианты нечем различить"
    );
    assert_eq!(
        overlay()["productName"],
        serde_json::json!("tolearn-with-speech"),
        "оверлей не переименовывает продукт — установщики варианта не отличить"
    );
}

#[test]
fn веса_едут_внутри_установщика() {
    let target =
        overlay()["bundle"]["resources"][format!("models/{}", tolearn_speech::model::FILE)].clone();

    assert_eq!(
        target,
        serde_json::json!(tolearn_speech::model::FILE),
        "модель `{}` не объявлена ресурсом: вариант соберётся без весов",
        tolearn_speech::model::FILE
    );
}

#[test]
fn ci_собирает_оба_варианта() {
    let ci = read(".github/workflows/ci.yml");
    let package = read(PACKAGE);

    assert!(
        ci.contains(&format!("uses: ./{PACKAGE}")),
        "CI не зовёт плиту сборки: установщики собираются мимо неё"
    );
    assert!(
        package.contains("variant: [base, with-speech]"),
        "в джобе `package` нет матрицы вариантов"
    );
    assert!(
        package
            .contains("cargo tauri build --features speech --config tauri.with-speech.conf.json"),
        "CI не собирает вариант с речью тем же флагом, что описан на странице релиза"
    );
}

#[test]
fn релиз_собирает_ту_же_матрицу_и_оставляет_черновик() {
    let release = read(RELEASE);

    assert!(
        release.contains(&format!("uses: ./{PACKAGE}")),
        "релиз собирает установщики своей копией матрицы, а не той же плитой, что CI"
    );
    assert!(
        release.contains("--draft"),
        "релиз публикуется сам: человеку нечего подтверждать"
    );
}

#[test]
fn страница_релиза_называет_оба_варианта() {
    for (name, _) in PAGES {
        let page = read(name);

        for variant in ["`tolearn`", "`tolearn-with-speech`"] {
            assert!(
                page.contains(variant),
                "на странице релиза {name} не назван вариант {variant}"
            );
        }
        assert!(
            page.contains(
                "cargo tauri build --features speech --config tauri.with-speech.conf.json"
            ),
            "страница релиза {name} не показывает, каким флагом собирается вариант с речью"
        );
    }
}

#[test]
fn страница_релиза_повторяет_потолки_из_бюджетов() {
    let budgets = read("docs/architecture.md");

    for row in [
        "| Установщик, macOS arm64 | ≤ 12 МБ |",
        "| Установщик, Windows x64 | ≤ 14 МБ |",
    ] {
        assert!(budgets.contains(row), "в бюджетах пропала строка `{row}`");
    }

    for (name, limits) in PAGES {
        let page = read(name);

        for limit in limits {
            assert!(
                page.contains(limit),
                "страница релиза {name} разошлась с бюджетом: нет потолка `{limit}`"
            );
        }
    }
}
