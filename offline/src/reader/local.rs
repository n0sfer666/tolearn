use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

const CAP: u64 = 4 * 1024 * 1024;

const TYPES: [(&str, &str); 8] = [
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
    ("bmp", "image/bmp"),
    ("svg", "image/svg+xml"),
];

pub fn inlined(src: &str, base: &Path) -> Option<String> {
    if src.starts_with("data:") {
        return Some(src.to_string());
    }
    let name = wanted(src)?;
    let file = base.join(name);
    if std::fs::metadata(&file).ok()?.len() > CAP {
        return None;
    }
    let bytes = std::fs::read(&file).ok()?;
    Some(format!(
        "data:{};base64,{}",
        kind(name),
        STANDARD.encode(bytes)
    ))
}

fn wanted(src: &str) -> Option<&str> {
    let path = src.split(['?', '#']).next()?.trim();
    let name = path.rsplit('/').next()?;
    if name.is_empty() || name == ".." || name == "." {
        return None;
    }
    Some(name)
}

fn kind(src: &str) -> &'static str {
    let tail = src.rsplit('.').next().unwrap_or_default().to_lowercase();
    TYPES
        .iter()
        .find(|(suffix, _)| *suffix == tail)
        .map(|(_, kind)| *kind)
        .unwrap_or("application/octet-stream")
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "offline gate: a panic here is the report"
)]
mod tests {
    use std::path::Path;

    use super::inlined;

    fn corner(name: &str) -> std::path::PathBuf {
        let corner =
            std::env::temp_dir().join(format!("tolearn-inline-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&corner).unwrap();
        std::fs::write(corner.join("scheme.png"), b"PNGBYTES").unwrap();
        corner
    }

    #[test]
    fn соседний_файл_становится_data_uri() {
        let uri = inlined("scheme.png", &corner("соседний")).unwrap();

        assert!(uri.starts_with("data:image/png;base64,"), "{uri}");
        assert!(uri.ends_with("UE5HQllURVM="), "{uri}");
    }

    #[test]
    fn готовый_data_uri_не_трогается() {
        let uri = inlined("data:image/gif;base64,AA==", Path::new("/nowhere")).unwrap();

        assert_eq!(uri, "data:image/gif;base64,AA==");
    }

    #[test]
    fn читается_только_соседний_файл() {
        assert!(inlined("../../etc/passwd", &corner("только-соседний")).is_none());
        assert!(inlined("/etc/hosts", &corner("только-соседний")).is_none());
    }

    #[test]
    fn у_абсолютной_ссылки_берётся_имя_файла() {
        let uri = inlined(
            "https://docs.test/img/scheme.png?v=2",
            &corner("абсолютная"),
        )
        .unwrap();

        assert!(uri.starts_with("data:image/png;base64,"), "{uri}");
    }

    #[test]
    fn отсутствующий_файл_молчит() {
        assert!(inlined("gone.png", &corner("пропавший")).is_none());
    }
}
