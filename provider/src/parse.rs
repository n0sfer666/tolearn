use tolearn_core::yaml::{ParseError, Reader, read};

use crate::legacy;
use crate::render::SCHEMA;
use crate::types::{
    API, Api, DEFAULT_TEMPERATURE_TENTHS, Harness, Http, KIND, NUM_CTX_MAX, Provider,
    TEMPERATURE_TENTHS_MAX,
};

pub const SCHEMA_V1: &str = "tolearn/provider/v1";

pub fn provider(source: &str) -> Result<Provider, ParseError> {
    read(source, |node| {
        let schema = node.field("schema")?;
        match schema.text()?.as_str() {
            SCHEMA_V1 => legacy::provider(node),
            SCHEMA => current(node),
            unknown => Err(schema.unknown(format!("unknown provider schema `{unknown}`"))),
        }
    })
}

fn current(node: &Reader<'_>) -> Result<Provider, ParseError> {
    Ok(Provider {
        enabled: node.field("enabled")?.flag()?,
        active: node.field("active")?.choice("kind", &KIND)?,
        local: http(&node.field("local")?, Api::Ollama)?,
        remote: http(&node.field("remote")?, Api::OpenAi)?,
        harness: harness(&node.field("harness")?)?,
        journal: match node.optional_field("journal")? {
            Some(field) => field.flag()?,
            None => false,
        },
    })
}

fn http(node: &Reader<'_>, spoken: Api) -> Result<Http, ParseError> {
    let api = match node.optional_field("api")? {
        Some(field) => field.choice("api", &API)?,
        None => spoken,
    };
    Ok(Http {
        endpoint: node.field("endpoint")?.any_text()?,
        api,
        model: node.field("model")?.any_text()?,
        num_ctx: tuned(node, "num_ctx", NUM_CTX_MAX, 0)?,
        temperature_tenths: tuned(
            node,
            "temperature_tenths",
            TEMPERATURE_TENTHS_MAX,
            DEFAULT_TEMPERATURE_TENTHS,
        )?,
    })
}

fn tuned(node: &Reader<'_>, name: &str, most: u32, spoken: u32) -> Result<u32, ParseError> {
    match node.optional_field(name)? {
        Some(field) => field.bounded(0, most),
        None => Ok(spoken),
    }
}

fn harness(node: &Reader<'_>) -> Result<Harness, ParseError> {
    Ok(Harness {
        id: node.field("id")?.any_text()?,
        command: node.field("command")?.any_text()?,
        args: node.field("args")?.list(Reader::any_text)?,
        timeout_secs: node.field("timeout_secs")?.bounded(1, 3_600)?,
    })
}
