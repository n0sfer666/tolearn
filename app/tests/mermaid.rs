#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "vendored mermaid gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tauri::http::{StatusCode, header};
use tolearn_app::mermaid::{POLICY, respond};
use tolearn_offline::digest::digest;

const VENDORED: &str = "app/vendor/mermaid/mermaid.tiny.js";
const LICENSE: &str = "app/vendor/mermaid/LICENSE";
const VERSION: &str = "`@mermaid-js/tiny` 12.0.0";

fn root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name)
}

fn read(name: &str) -> Vec<u8> {
    let path = root(name);
    match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(reason) => panic!("{}: {reason}", path.display()),
    }
}

fn notice() -> String {
    String::from_utf8(read("THIRD-PARTY.md")).unwrap()
}

#[test]
fn the_vendored_mermaid_is_the_build_the_notice_names() {
    let notice = notice();
    let hash = digest(&read(VENDORED));
    assert!(
        notice.contains(&hash),
        "THIRD-PARTY.md не называет sha256 {hash} файла {VENDORED}"
    );
    assert!(notice.contains(VERSION), "THIRD-PARTY.md без {VERSION}");
    assert!(
        notice.contains(VENDORED),
        "THIRD-PARTY.md без пути {VENDORED}"
    );
}

#[test]
fn the_mermaid_license_ships_next_to_it() {
    let license = String::from_utf8(read(LICENSE)).unwrap();
    assert!(license.contains("MIT"), "{LICENSE} не MIT");
    assert!(
        license.contains("Knut Sveidqvist"),
        "{LICENSE} без правообладателя"
    );
}

#[test]
fn the_scheme_serves_the_page_the_script_and_mermaid_itself() {
    for (path, kind) in [
        ("/", "text/html"),
        ("/index.html", "text/html"),
        ("/draw.js", "text/javascript"),
        ("/mermaid.tiny.js", "text/javascript"),
    ] {
        let answer = respond(path);
        assert_eq!(answer.status(), StatusCode::OK, "{path}");
        let served = answer.headers()[header::CONTENT_TYPE].to_str().unwrap();
        assert!(served.starts_with(kind), "{path}: {served}");
        assert!(!answer.body().is_empty(), "{path}: пустой ответ");
    }
    assert_eq!(respond("/mermaid.tiny.js").body().as_ref(), read(VENDORED));
}

#[test]
fn the_page_loads_nothing_but_its_own_scripts() {
    assert!(POLICY.starts_with("default-src 'none'"), "{POLICY}");
    assert!(POLICY.contains("script-src 'self'"), "{POLICY}");
    assert!(!POLICY.contains("unsafe-eval"), "{POLICY}");
    for path in ["/", "/draw.js", "/mermaid.tiny.js", "/missing"] {
        let answer = respond(path);
        let policy = answer.headers()[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap();
        assert_eq!(policy, POLICY, "{path}");
    }
    let index = String::from_utf8(respond("/").body().to_vec()).unwrap();
    assert!(
        !index.contains("http"),
        "страница схемы ссылается наружу: {index}"
    );
}

#[test]
fn the_draw_script_keeps_mermaid_strict_and_labels_plain() {
    let script = String::from_utf8(respond("/draw.js").body().to_vec()).unwrap();
    for needle in [
        "securityLevel: \"strict\"",
        "htmlLabels: false",
        "startOnLoad: false",
        "XMLSerializer",
    ] {
        assert!(script.contains(needle), "draw.js без {needle}");
    }
}

#[test]
fn an_unknown_path_is_not_found() {
    assert_eq!(respond("/secret").status(), StatusCode::NOT_FOUND);
    assert!(respond("/secret").body().is_empty());
}
