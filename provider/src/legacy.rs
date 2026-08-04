use tolearn_core::yaml::{ParseError, Reader};

use crate::types::{Api, Http, Kind, Provider};

pub fn provider(node: &Reader<'_>) -> Result<Provider, ParseError> {
    let flavor = node.field("flavor")?;
    let endpoint = node.field("endpoint")?.text()?;
    let model = node.field("model")?.any_text()?;
    let enabled = node.field("enabled")?.flag()?;
    let spoken = |api| Http {
        endpoint: endpoint.clone(),
        api,
        model: model.clone(),
    };

    match flavor.text()?.as_str() {
        "ollama" => Ok(Provider {
            enabled,
            active: Kind::Local,
            local: spoken(Api::Ollama),
            ..Provider::default()
        }),
        "openai" => Ok(Provider {
            enabled,
            active: Kind::Remote,
            remote: spoken(Api::OpenAi),
            ..Provider::default()
        }),
        unknown => Err(flavor.unknown(format!("unknown flavor `{unknown}`"))),
    }
}
