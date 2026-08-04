use tolearn_core::yaml::{ParseError, Reader};

use crate::types::{Http, Kind, Provider};

pub fn provider(node: &Reader<'_>) -> Result<Provider, ParseError> {
    let flavor = node.field("flavor")?;
    let http = Http {
        endpoint: node.field("endpoint")?.text()?,
        model: node.field("model")?.any_text()?,
    };
    let enabled = node.field("enabled")?.flag()?;

    match flavor.text()?.as_str() {
        "ollama" => Ok(Provider {
            enabled,
            active: Kind::Local,
            local: http,
            ..Provider::default()
        }),
        "openai" => Ok(Provider {
            enabled,
            active: Kind::Remote,
            remote: http,
            ..Provider::default()
        }),
        unknown => Err(flavor.unknown(format!("unknown flavor `{unknown}`"))),
    }
}
