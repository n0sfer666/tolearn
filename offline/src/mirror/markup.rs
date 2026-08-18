use pulldown_cmark::{Options, Parser, html};

const STRUCTURE: [&str; 3] = ["<!doctype", "<html", "<body"];

pub(super) fn as_html(text: &str, title: &str) -> Option<String> {
    if !markdown(text) {
        return None;
    }
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    let mut body = String::new();
    html::push_html(&mut body, Parser::new_ext(text, options));
    Some(format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title></head><body>{body}</body></html>",
        escaped(title)
    ))
}

fn markdown(text: &str) -> bool {
    let head: String = text.chars().take(4096).collect::<String>().to_lowercase();
    !STRUCTURE.iter().any(|tag| head.contains(tag))
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "offline gate: a panic here is the report"
)]
mod tests {
    use super::as_html;

    #[test]
    fn страница_с_разметкой_html_не_трогается() {
        assert!(
            as_html(
                "<!doctype html><html><body><h1>Есть</h1></body></html>",
                "т"
            )
            .is_none()
        );
    }

    #[test]
    fn markdown_становится_разметкой() {
        let html = as_html("# Длина контекста\n\nТекст со `ссылкой`.\n", "Длина").unwrap();

        assert!(html.contains("<h1>Длина контекста</h1>"), "заголовок сырой");
        assert!(html.contains("<code>ссылкой</code>"), "код сырой");
        assert!(html.contains("<title>Длина</title>"), "нет заголовка окна");
    }

    #[test]
    fn фенса_кода_превращается_в_блок() {
        let html = as_html("Пример:\n\n```yaml\nkey: 1\n```\n", "т").unwrap();

        assert!(html.contains("<pre><code"), "фенса осталась текстом");
    }

    #[test]
    fn заголовок_окна_экранируется() {
        let html = as_html("# А\n", "<b>&").unwrap();

        assert!(
            html.contains("<title>&lt;b&gt;&amp;</title>"),
            "заголовок не экранирован"
        );
    }
}
