use std::time::Duration;

use serde_json::json;

use crate::error::CheckError;
use crate::harness;
use crate::types::{Api, Http, Kind, Provider};
use crate::wire::{apart, broken, client, given, refused};

pub const PATIENCE: Duration = Duration::from_secs(180);

pub fn ask(provider: &Provider, key: Option<&str>, prompt: &str) -> Result<String, CheckError> {
    if !provider.enabled {
        return Err(CheckError::Disabled);
    }
    match provider.active {
        Kind::Harness => harness::ask(&provider.harness, prompt),
        Kind::Local => spoken(&provider.local, None, prompt),
        Kind::Remote => {
            let key = given(key).ok_or(CheckError::NoKey)?;
            spoken(&provider.remote, Some(key), prompt)
        }
    }
}

fn spoken(http: &Http, key: Option<&str>, prompt: &str) -> Result<String, CheckError> {
    if http.model.trim().is_empty() {
        return Err(CheckError::NoModel);
    }
    let asked = http.clone();
    let key = key.map(str::to_owned);
    let prompt = prompt.to_owned();
    apart(move || send(&asked, key.as_deref(), &prompt))
}

fn send(http: &Http, key: Option<&str>, prompt: &str) -> Result<String, CheckError> {
    let mut request = client(PATIENCE)?
        .post(route(http))
        .header("content-type", "application/json")
        .body(body(http, prompt).to_string());
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }

    let answer = request.send().map_err(broken)?;
    if let Some(refused) = refused(answer.status().as_u16()) {
        return Err(refused);
    }

    let body = answer.text().map_err(broken)?;
    said(http.api, &body).ok_or(CheckError::BadAnswer)
}

fn route(http: &Http) -> String {
    let base = http.endpoint.trim_end_matches('/');
    match http.api {
        Api::Ollama => format!("{base}/api/chat"),
        Api::OpenAi => format!("{base}/chat/completions"),
    }
}

fn body(http: &Http, prompt: &str) -> serde_json::Value {
    json!({
        "model": http.model,
        "messages": [{ "role": "user", "content": prompt }],
        "stream": false,
    })
}

fn said(api: Api, body: &str) -> Option<String> {
    let answer: serde_json::Value = serde_json::from_str(body).ok()?;
    let said = match api {
        Api::Ollama => answer.get("message")?.get("content")?,
        Api::OpenAi => answer
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
