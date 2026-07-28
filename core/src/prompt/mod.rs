mod error;
mod generation;
mod values;

pub use error::RenderError;
pub use generation::generation;

use crate::roadmap::Roadmap;
use crate::topic::Topic;

use values::{Value, value};

const RULE: &str = "---";

pub fn render(template: &str, topic: &Topic, roadmap: &Roadmap) -> Result<String, RenderError> {
    let mut lines = Vec::new();
    for line in body(template)?.lines() {
        if let Some(filled) = fill(line, topic, roadmap)? {
            lines.push(filled);
        }
    }
    Ok(format!("{}\n", lines.join("\n").trim()))
}

fn body(template: &str) -> Result<String, RenderError> {
    let mut parts = template.split('\n').collect::<Vec<_>>();
    let rules: Vec<usize> = parts
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == RULE)
        .map(|(at, _)| at)
        .collect();
    let [first, second, ..] = rules.as_slice() else {
        return Err(RenderError::NoPrompt);
    };
    Ok(parts
        .drain(first + 1..*second)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn fill(line: &str, topic: &Topic, roadmap: &Roadmap) -> Result<Option<String>, RenderError> {
    let indent = " ".repeat(line.len() - line.trim_start().len());
    let mut head = String::new();
    let mut nested = Vec::new();
    let mut rest = line;

    while let Some(start) = rest.find("{{") {
        let Some(length) = rest[start..].find("}}") else {
            return Err(RenderError::Unclosed {
                line: line.to_owned(),
            });
        };
        let name = &rest[start + 2..start + length];
        let Some(found) = value(name, topic, roadmap) else {
            return Err(RenderError::Unknown {
                name: name.to_owned(),
            });
        };
        head.push_str(&rest[..start]);
        match found {
            Value::Scalar(text) if text.trim().is_empty() => return Ok(None),
            Value::Scalar(text) => head.push_str(&inline(&text, &indent)),
            Value::List(items) if items.is_empty() => return Ok(None),
            Value::List(items) => nested.extend(list(&items, &indent)),
        }
        rest = &rest[start + length + 2..];
    }
    head.push_str(rest);

    let head = head.trim_end();
    if nested.is_empty() {
        return Ok(Some(head.to_owned()));
    }
    if head.trim().is_empty() {
        return Ok(Some(nested.join("\n")));
    }
    Ok(Some(format!("{head}\n{}", nested.join("\n"))))
}

fn list(items: &[String], indent: &str) -> Vec<String> {
    items
        .iter()
        .map(|item| {
            let mut lines = item.trim().lines();
            let first = lines.next().unwrap_or_default();
            let mut block = format!("{indent}  - {first}");
            for line in lines {
                if line.trim().is_empty() {
                    block.push('\n');
                    continue;
                }
                block.push_str(&format!("\n{indent}  {line}"));
            }
            block
        })
        .collect()
}

fn inline(text: &str, indent: &str) -> String {
    text.trim()
        .lines()
        .map(str::trim_end)
        .enumerate()
        .map(|(at, line)| match (at, line.is_empty()) {
            (0, _) | (_, true) => line.to_owned(),
            _ => format!("{indent}  {line}"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
