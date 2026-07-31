use markup5ever_rcdom::{Handle, NodeData};
use monolith::html::{get_node_attr, get_node_name, html_to_dom};

const BOXED: [(&str, Kind); 11] = [
    ("h1", Kind::Heading),
    ("h2", Kind::Heading),
    ("h3", Kind::Heading),
    ("h4", Kind::Heading),
    ("h5", Kind::Heading),
    ("h6", Kind::Heading),
    ("p", Kind::Paragraph),
    ("li", Kind::Item),
    ("dd", Kind::Item),
    ("blockquote", Kind::Quote),
    ("pre", Kind::Code),
];

const MUTE: [&str; 4] = ["script", "style", "noscript", "template"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Heading,
    Paragraph,
    Item,
    Quote,
    Code,
    Image,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Heading => "heading",
            Self::Paragraph => "paragraph",
            Self::Item => "item",
            Self::Quote => "quote",
            Self::Code => "code",
            Self::Image => "image",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub kind: Kind,
    pub level: u32,
    pub text: String,
    pub src: String,
}

pub fn pieces(html: &str) -> Vec<Piece> {
    let dom = html_to_dom(&html.as_bytes().to_vec(), "utf-8".to_string());
    let mut found = Vec::new();
    scan(&dom.document, &mut found);
    found
}

fn scan(node: &Handle, found: &mut Vec<Piece>) {
    let name = get_node_name(node).unwrap_or_default();
    if MUTE.contains(&name) {
        return;
    }
    if name == "img" {
        found.extend(pictured(node));
        return;
    }
    let Some(kind) = BOXED
        .iter()
        .find(|(tag, _)| *tag == name)
        .map(|(_, kind)| *kind)
    else {
        for child in node.children.borrow().iter() {
            scan(child, found);
        }
        return;
    };
    let text = worded(node, kind == Kind::Code);
    if !text.is_empty() {
        found.push(Piece {
            kind,
            level: deep(name),
            text,
            src: String::new(),
        });
    }
    inside(node, found);
}

fn inside(node: &Handle, found: &mut Vec<Piece>) {
    if get_node_name(node) == Some("img") {
        found.extend(pictured(node));
        return;
    }
    for child in node.children.borrow().iter() {
        inside(child, found);
    }
}

fn pictured(node: &Handle) -> Option<Piece> {
    let src = get_node_attr(node, "src")
        .or_else(|| get_node_attr(node, "data-src"))
        .unwrap_or_default();
    if src.trim().is_empty() {
        return None;
    }
    Some(Piece {
        kind: Kind::Image,
        level: 0,
        text: get_node_attr(node, "alt").unwrap_or_default(),
        src: src.trim().to_string(),
    })
}

fn worded(node: &Handle, exact: bool) -> String {
    let mut text = String::new();
    gather(node, &mut text);
    if exact {
        return text.trim_matches('\n').to_string();
    }
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn gather(node: &Handle, into: &mut String) {
    if MUTE.contains(&get_node_name(node).unwrap_or_default()) {
        return;
    }
    if let NodeData::Text { contents } = &node.data {
        into.push_str(&contents.borrow());
    }
    for child in node.children.borrow().iter() {
        gather(child, into);
    }
}

fn deep(name: &str) -> u32 {
    name.strip_prefix('h')
        .and_then(|rank| rank.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    reason = "offline gate: a panic here is the report"
)]
mod tests {
    use super::{Kind, pieces};

    #[test]
    fn заголовки_несут_уровень() {
        let found = pieces("<h1>Раз</h1><h3>Три</h3>");

        assert_eq!(found[0].kind, Kind::Heading);
        assert_eq!(found[0].level, 1);
        assert_eq!(found[1].level, 3);
    }

    #[test]
    fn картинка_становится_куском() {
        let found = pieces("<p>До</p><img src=\"scheme.png\" alt=\"Схема\">");

        assert_eq!(found[1].kind, Kind::Image);
        assert_eq!(found[1].src, "scheme.png");
        assert_eq!(found[1].text, "Схема");
    }

    #[test]
    fn картинка_внутри_абзаца_не_теряется() {
        let found = pieces("<p>Текст <img src=\"a.png\"></p>");

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].text, "Текст");
        assert_eq!(found[1].kind, Kind::Image);
    }

    #[test]
    fn код_сохраняет_переводы_строк() {
        let found = pieces("<pre><code>a = 1\nb = 2</code></pre>");

        assert_eq!(found[0].kind, Kind::Code);
        assert_eq!(found[0].text, "a = 1\nb = 2");
    }

    #[test]
    fn цитата_собирается_одним_куском() {
        let found = pieces("<blockquote><p>Слова</p></blockquote>");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, Kind::Quote);
    }

    #[test]
    fn скрипты_и_пустое_не_попадают() {
        let found = pieces("<script>var a = 1;</script><p>  </p><img alt=\"нет\">");

        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn пункты_списка_идут_подряд() {
        let found = pieces("<ul><li>Раз</li><li>Два</li></ul>");

        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|piece| piece.kind == Kind::Item));
    }
}
