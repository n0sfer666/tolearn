#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "manifest gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

const LOCALES: [(&str, &str); 2] = [("ru", "en"), ("en", "ru")];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn markdown(dir: &Path, found: &mut Vec<PathBuf>) {
    let listing = match std::fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(reason) => panic!("{}: {reason}", dir.display()),
    };

    for entry in listing {
        let path = entry.unwrap().path();

        if path.is_dir() {
            markdown(&path, found);
        } else if path.extension().is_some_and(|kind| kind == "md") {
            found.push(path);
        }
    }
}

fn url(page: &Path) -> String {
    page.components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn pages(locale: &str) -> Vec<PathBuf> {
    let base = root().join("docs").join(locale);
    let mut found = Vec::new();
    markdown(&base, &mut found);
    found
        .into_iter()
        .map(|path| path.strip_prefix(&base).unwrap().to_path_buf())
        .collect()
}

fn links(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("](") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find(')') else { break };
        found.push(rest[..end].to_owned());
        rest = &rest[end..];
    }

    found
}

#[test]
fn деревья_локалей_зеркальны() {
    for (locale, twin) in LOCALES {
        for page in pages(locale) {
            assert!(
                root().join("docs").join(twin).join(&page).exists(),
                "у docs/{locale}/{} нет пары в docs/{twin}",
                url(&page)
            );
        }
    }
}

#[test]
fn каждая_страница_ведёт_на_свою_пару() {
    for (locale, twin) in LOCALES {
        for page in pages(locale) {
            let path = root().join("docs").join(locale).join(&page);
            let text = std::fs::read_to_string(&path).unwrap();
            let depth = page.components().count();
            let up = "../".repeat(depth);
            let target = format!("({up}{twin}/{})", url(&page));

            assert!(
                text.contains(&target),
                "docs/{locale}/{} не ссылается на свою пару `{target}`",
                url(&page)
            );
        }
    }
}

#[test]
fn ссылки_в_документации_никуда_не_ведут() {
    let mut files = vec![root().join("README.md"), root().join("README.ru.md")];
    markdown(&root().join("docs"), &mut files);

    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();

        for link in links(&text) {
            if link.contains("://") || link.starts_with('#') || link.starts_with("mailto:") {
                continue;
            }

            let target = link.split('#').next().unwrap();

            if target.is_empty() {
                continue;
            }

            let resolved = file.parent().unwrap().join(target);

            assert!(
                resolved.exists(),
                "{}: ссылка `{link}` ведёт в пустоту",
                file.strip_prefix(root()).unwrap().display()
            );
        }
    }
}
