use std::fmt::Write;

use saphyr::{Scalar, Yaml, YamlEmitter};

use super::error::DocumentError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Yaml,
    Json,
}

pub fn render(root: &Yaml<'_>, format: Format) -> Result<String, DocumentError> {
    let mut out = String::new();
    match format {
        Format::Yaml => {
            let mut emitter = YamlEmitter::new(&mut out);
            emitter.multiline_strings(true);
            emitter
                .dump(root)
                .map_err(|error| DocumentError::Malformed(format!("{error:?}")))?;
            out = out.strip_prefix("---\n").unwrap_or(&out).to_owned();
        }
        Format::Json => json(root, &mut out)?,
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn json(node: &Yaml<'_>, out: &mut String) -> Result<(), DocumentError> {
    match node {
        Yaml::Value(scalar) => scalar_json(scalar, out),
        Yaml::Representation(value, style, tag) => {
            let scalar =
                Scalar::parse_from_cow_and_metadata(value.clone(), *style, tag.as_ref())
                    .ok_or_else(|| DocumentError::Malformed(format!("`{value}` is no scalar")))?;
            scalar_json(&scalar, out)
        }
        Yaml::Sequence(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                json(item, out)?;
            }
            out.push(']');
            Ok(())
        }
        Yaml::Mapping(fields) => {
            out.push('{');
            for (index, (key, value)) in fields.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                let name = key.as_str().ok_or_else(|| {
                    DocumentError::Malformed("a key that is not a string".to_owned())
                })?;
                quoted(name, out);
                out.push(':');
                json(value, out)?;
            }
            out.push('}');
            Ok(())
        }
        other => Err(DocumentError::Malformed(format!(
            "{other:?} has no place in a progress file"
        ))),
    }
}

fn scalar_json(scalar: &Scalar<'_>, out: &mut String) -> Result<(), DocumentError> {
    match scalar {
        Scalar::Null => out.push_str("null"),
        Scalar::Boolean(value) => {
            let _ = write!(out, "{value}");
        }
        Scalar::Integer(value) => {
            let _ = write!(out, "{value}");
        }
        Scalar::FloatingPoint(value) => {
            let _ = write!(out, "{value}");
        }
        Scalar::String(value) => quoted(value, out),
    }
    Ok(())
}

fn quoted(value: &str, out: &mut String) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control < ' ' => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            plain => out.push(plain),
        }
    }
    out.push('"');
}
