use crate::block::{Block, Kind};

use super::escape::{info, local};
use super::lines::{Doc, label, longest};
use super::prose::prose;
use super::words::Words;

pub fn blocks(doc: &mut Doc, blocks: &[Block], depth: usize, words: &Words) {
    for block in blocks {
        match block.kind {
            Kind::Heading => doc.heading(depth, &block.text),
            Kind::Paragraph => doc.raw(prose(&block.text, depth)),
            Kind::Callout => doc.raw(quoted(prose(&block.text, depth))),
            Kind::Code => doc.raw(fenced(&block.text, block.lang.as_deref())),
            Kind::Diagram => picture(doc, words.diagram, block),
            Kind::Image => {
                picture(doc, &block.text, block);
                let credits = credits(block, words);
                if !credits.is_empty() {
                    doc.line(&credits);
                }
            }
        }
    }
}

fn quoted(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| {
            if line.is_empty() {
                ">".to_owned()
            } else {
                format!("> {line}")
            }
        })
        .collect()
}

fn fenced(text: &str, lang: Option<&str>) -> Vec<String> {
    let fence = "`".repeat(longest(text).max(2) + 1);
    let mut lines = vec![format!("{fence}{}", info(lang))];
    lines.extend(text.lines().map(|line| line.trim_end().to_owned()));
    lines.push(fence);
    lines
}

fn picture(doc: &mut Doc, alt: &str, block: &Block) {
    if let Some(asset) = &block.asset {
        doc.raw(vec![format!("![{}]({})", label(alt), local(asset))]);
    }
}

fn credits(block: &Block, words: &Words) -> String {
    [
        (words.license, &block.license),
        (words.author, &block.attribution),
        (words.source, &block.source),
    ]
    .into_iter()
    .filter_map(|(word, value)| value.as_deref().map(|value| format!("{word}: {value}")))
    .collect::<Vec<String>>()
    .join(" · ")
}
