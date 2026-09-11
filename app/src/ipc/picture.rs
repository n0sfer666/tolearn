use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub fn uri(file: &str, bytes: &[u8]) -> String {
    format!("data:{};base64,{}", mime(file), STANDARD.encode(bytes))
}

fn mime(file: &str) -> &'static str {
    let extension = file
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::uri;

    #[test]
    fn the_extension_names_the_type_in_any_case() {
        for (file, mime) in [
            ("assets/a.SVG", "image/svg+xml"),
            ("assets/b.Png", "image/png"),
            ("assets/c.jpeg", "image/jpeg"),
            ("assets/d.JPG", "image/jpeg"),
            ("assets/e.gif", "image/gif"),
            ("assets/f.webp", "image/webp"),
            ("assets/g.bin", "application/octet-stream"),
        ] {
            assert!(
                uri(file, b"x").starts_with(&format!("data:{mime};base64,")),
                "{file}"
            );
        }
    }
}
