use std::collections::BTreeMap;
use std::fmt::Write;

use serde_json::Value;

use super::SCHEMA;
use super::unpack_error::UnpackError;

pub(super) fn read(data: &[u8]) -> Result<BTreeMap<String, String>, UnpackError> {
    let value: Value = serde_json::from_slice(data)
        .map_err(|error| malformed(format!("it is not JSON: {error}")))?;
    let Value::Object(mut fields) = value else {
        return Err(malformed("it is not a JSON object"));
    };
    match fields.remove("schema") {
        Some(Value::String(found)) if found == SCHEMA => {}
        Some(Value::String(found)) => return Err(UnpackError::Version { found }),
        Some(_) => return Err(malformed("`schema` is not a string")),
        None => return Err(malformed("it has no `schema`")),
    }
    let files = fields.remove("files");
    if let Some(field) = fields.keys().next() {
        return Err(malformed(format!("`{field}` is not a field of a manifest")));
    }
    let Some(Value::Object(files)) = files else {
        return Err(malformed("`files` is not an object"));
    };
    files
        .into_iter()
        .map(|(file, sum)| match sum {
            Value::String(sum) if is_sum(&sum) => Ok((file, sum)),
            _ => Err(malformed(format!("the sum of `{file}` is not a sha256"))),
        })
        .collect()
}

fn malformed(reason: impl Into<String>) -> UnpackError {
    UnpackError::BadManifest {
        reason: reason.into(),
    }
}

fn is_sum(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn render(sums: &[(String, String)]) -> String {
    let files: Vec<String> = sums
        .iter()
        .map(|(name, sum)| format!("    {}: {}", quoted(name), quoted(sum)))
        .collect();
    format!(
        "{{\n  \"schema\": {},\n  \"files\": {{\n{}\n  }}\n}}\n",
        quoted(SCHEMA),
        files.join(",\n")
    )
}

fn quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for symbol in text.chars() {
        match symbol {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            symbol if symbol < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(symbol));
            }
            symbol => out.push(symbol),
        }
    }
    out.push('"');
    out
}
