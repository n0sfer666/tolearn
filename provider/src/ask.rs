use std::time::Duration;

use serde_json::json;

use crate::error::CheckError;
use crate::types::{Flavor, Provider};
use crate::wire::{apart, broken, client, refused};

pub const PATIENCE: Duration = Duration::from_secs(180);

pub fn ask(provider: &Provider, key: Option<&str>, prompt: &str) -> Result<String, CheckError> {
    if !provider.enabled {
        return Err(CheckError::Disabled);
    }
    if provider.model.trim().is_empty() {
        return Err(CheckError::NoModel);
    }
    let key = key.map(str::trim).filter(|key| !key.is_empty());
    if provider.flavor == Flavor::OpenAi && key.is_none() {
        return Err(CheckError::NoKey);
    }

    let asked = provider.clone();
    let key = key.map(str::to_owned);
    let prompt = prompt.to_owned();
    apart(move || send(&asked, key.as_deref(), &prompt))
}

fn send(provider: &Provider, key: Option<&str>, prompt: &str) -> Result<String, CheckError> {
    let mut request = client(PATIENCE)?
        .post(route(provider))
        .header("content-type", "application/json")
        .body(body(provider, prompt).to_string());
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }

    let answer = request.send().map_err(broken)?;
    if let Some(refused) = refused(answer.status().as_u16()) {
        return Err(refused);
    }

    let body = answer.text().map_err(broken)?;
    said(provider.flavor, &body).ok_or(CheckError::BadAnswer)
}

fn route(provider: &Provider) -> String {
    let base = provider.endpoint.trim_end_matches('/');
    match provider.flavor {
        Flavor::Ollama => format!("{base}/api/chat"),
        Flavor::OpenAi => format!("{base}/chat/completions"),
    }
}

fn body(provider: &Provider, prompt: &str) -> serde_json::Value {
    let messages = json!([{ "role": "user", "content": prompt }]);
    match provider.flavor {
        Flavor::Ollama => json!({
            "model": provider.model,
            "messages": messages,
            "stream": false,
        }),
        Flavor::OpenAi => json!({
            "model": provider.model,
            "messages": messages,
        }),
    }
}

fn said(flavor: Flavor, body: &str) -> Option<String> {
    let answer: serde_json::Value = serde_json::from_str(body).ok()?;
    let said = match flavor {
        Flavor::Ollama => answer.get("message")?.get("content")?,
        Flavor::OpenAi => answer
            .get("choices")?
            .as_array()?
            .first()?
            .get("message")?
            .get("content")?,
    };
    let said = said.as_str()?.trim();
    match said.is_empty() {
        true => None,
        false => Some(said.to_owned()),
    }
}
