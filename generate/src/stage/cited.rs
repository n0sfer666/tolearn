use tolearn_core::block::{Block, Kind};

use super::flaw::Flaw;
use super::gathered::Gathered;

const SCHEMES: [&str; 2] = ["https://", "http://"];
const TRAILING: &[char] = &['.', ',', ';', ':', '!', '?', '\''];

pub(super) fn cited(block: &Block, sources: &[String], gathered: &Gathered, flaws: &mut Vec<Flaw>) {
    for source in sources {
        if !gathered.knows(source) {
            flaws.push(Flaw::UnknownSource {
                block: block.id.clone(),
                source: source.clone(),
            });
        }
    }
    if matches!(block.kind, Kind::Code | Kind::Diagram) {
        return;
    }
    for url in links(&block.text) {
        if !gathered.visited(url) {
            flaws.push(Flaw::ForeignLink {
                block: block.id.clone(),
                url: url.to_owned(),
            });
        }
    }
}

fn links(text: &str) -> Vec<&str> {
    text.match_indices("http")
        .filter_map(|(start, _)| {
            let rest = &text[start..];
            SCHEMES
                .iter()
                .any(|scheme| rest.starts_with(scheme))
                .then(|| {
                    let end = rest
                        .find(|c: char| {
                            c.is_whitespace() || matches!(c, ')' | ']' | '>' | '<' | '"' | '»')
                        })
                        .unwrap_or(rest.len());
                    rest[..end].trim_end_matches(TRAILING)
                })
        })
        .collect()
}
