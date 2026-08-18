use super::error::LinkError;

const SCHEME: &str = "tolearn://";
const ACTION: &str = "topic";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub roadmap: String,
    pub topic: String,
}

pub fn parse(url: &str) -> Result<Link, LinkError> {
    let rest = url
        .trim()
        .strip_prefix(SCHEME)
        .ok_or_else(|| LinkError::Scheme {
            given: url.trim().to_owned(),
        })?;
    let (action, query) = rest.split_once('?').unwrap_or((rest, ""));
    if action != ACTION {
        return Err(LinkError::Action {
            given: action.to_owned(),
        });
    }

    let mut roadmap = None;
    let mut topic = None;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let slot = match name {
            "roadmap" => &mut roadmap,
            "topic" => &mut topic,
            other => {
                return Err(LinkError::Extra {
                    field: other.to_owned(),
                });
            }
        };
        if slot.is_some() {
            return Err(LinkError::Extra {
                field: name.to_owned(),
            });
        }
        *slot = Some(identifier(name, value)?);
    }

    Ok(Link {
        roadmap: taken("roadmap", roadmap)?,
        topic: taken("topic", topic)?,
    })
}

fn taken(field: &str, value: Option<String>) -> Result<String, LinkError> {
    value.ok_or_else(|| LinkError::Missing {
        field: field.to_owned(),
    })
}

fn identifier(field: &str, raw: &str) -> Result<String, LinkError> {
    let value = decoded(field, raw)?;
    if value.is_empty() {
        return Err(LinkError::Missing {
            field: field.to_owned(),
        });
    }
    let allowed = value
        .chars()
        .all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '-' | '_' | '.'));
    if !allowed || value.starts_with('.') {
        return Err(LinkError::Value {
            field: field.to_owned(),
            given: value,
        });
    }
    Ok(value)
}

fn decoded(field: &str, raw: &str) -> Result<String, LinkError> {
    let mut out = String::with_capacity(raw.len());
    let mut letters = raw.chars();
    while let Some(letter) = letters.next() {
        if letter != '%' {
            out.push(letter);
            continue;
        }
        let digits: String = letters.by_ref().take(2).collect();
        let byte = u8::from_str_radix(&digits, 16).map_err(|_| LinkError::Value {
            field: field.to_owned(),
            given: raw.to_owned(),
        })?;
        out.push(char::from(byte));
    }
    Ok(out)
}
