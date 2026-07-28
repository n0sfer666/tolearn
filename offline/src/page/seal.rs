use markup5ever_rcdom::Handle;
use monolith::core::Options;
use monolith::html::{
    get_node_attr, get_node_name, html_to_dom, serialize_document, set_node_attr,
};

const LOADED: [&str; 5] = ["src", "srcset", "poster", "data-src", "href"];
const NAVIGATING: [&str; 2] = ["a", "base"];

pub(super) fn seal(html: Vec<u8>) -> Vec<u8> {
    let dom = html_to_dom(&html, "utf-8".to_string());
    strip(&dom.document);
    serialize_document(dom, "utf-8".to_string(), &Options::default())
}

fn strip(node: &Handle) {
    let navigating = get_node_name(node).is_some_and(|name| NAVIGATING.contains(&name));
    if !navigating {
        for attribute in LOADED {
            if remote(get_node_attr(node, attribute).as_deref()) {
                set_node_attr(node, attribute, None);
            }
        }
    }
    for child in node.children.borrow().iter() {
        strip(child);
    }
}

fn remote(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        let value = value.trim().to_ascii_lowercase();
        value.starts_with("http://") || value.starts_with("https://") || value.starts_with("//")
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "offline gate: a panic here is the report"
)]
mod tests {
    use super::seal;

    fn sealed(body: &str) -> String {
        String::from_utf8(seal(
            format!("<html><body>{body}</body></html>").into_bytes(),
        ))
        .unwrap()
    }

    #[test]
    fn ссылка_без_протокола_тоже_считается_внешней() {
        let html = sealed("<img src=\"//cdn.a.test/x.png\" alt=\"x\">");

        assert!(
            !html.contains("cdn.a.test"),
            "остался protocol-relative src"
        );
        assert!(html.contains("<img"), "узел картинки выброшен целиком");
    }

    #[test]
    fn встроенные_данные_не_трогаются() {
        let html = sealed("<img src=\"data:image/png;base64,AA\" alt=\"x\">");

        assert!(
            html.contains("data:image/png"),
            "встроенная картинка стёрта"
        );
    }

    #[test]
    fn заглавные_буквы_в_схеме_не_спасают() {
        let html = sealed("<img src=\"HTTPS://a.test/x.png\" alt=\"x\">");

        assert!(!html.contains("a.test"), "схема в верхнем регистре прошла");
    }
}
