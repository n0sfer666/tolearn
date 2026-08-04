use std::time::Duration;

use serde_json::json;

use crate::error::CheckError;
use crate::harness;
use crate::types::{Http, Kind, Provider};
use crate::wire::{apart, broken, client, given, refused};

pub const PATIENCE: Duration = Duration::from_secs(180);

pub fn ask(provider: &Provider, key: Option<&str>, prompt: &str) -> Result<String, CheckError> {
    if !provider.enabled {
        return Err(CheckError::Disabled);
    }
    match provider.active {
        Kind::Harness => harness::ask(&provider.harness, prompt),
        Kind::Local => spoken(&provider.local, None, Kind::Local, prompt),
        Kind::Remote => {
            let key = given(key).ok_or(CheckError::NoKey)?;
            spoken(&provider.remote, Some(key), Kind::Remote, prompt)
        }
    }
}

fn spoken(http: &Http, key: Option<&str>, kind: Kind, prompt: &str) -> Result<String, CheckError> {
    if http.model.trim().is_empty() {
        return Err(CheckError::NoModel);
    }
    let asked = http.clone();
    let key = key.map(str::to_owned);
    let prompt = prompt.to_owned();
    apart(move || send(&asked, key.as_deref(), kind, &prompt))
}

fn send(http: &Http, key: Option<&str>, kind: Kind, prompt: &str) -> Result<String, CheckError> {
    let mut request = client(PATIENCE)?
        .post(route(http, kind))
        .header("content-type", "application/json")
        .body(body(http, kind, prompt).to_string());
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }

    let answer = request.send().map_err(broken)?;
    if let Some(refused) = refused(answer.status().as_u16()) {
        return Err(refused);
    }

    let body = answer.text().map_err(broken)?;
    said(kind, &body).ok_or(CheckError::BadAnswer)
}

fn route(http: &Http, kind: Kind) -> String {
    let base = http.endpoint.trim_end_matches('/');
    match kind {
        Kind::Remote => format!("{base}/chat/completions"),
        _ => format!("{base}/api/chat"),
    }
}

fn body(http: &Http, kind: Kind, prompt: &str) -> serde_json::Value {
    let messages = json!([{ "role": "user", "content": prompt }]);
    match kind {
        Kind::Remote => json!({
            "model": http.model,
            "messages": messages,
        }),
        _ => json!({
            "model": http.model,
            "messages": messages,
            "stream": false,
        }),
    }
}

fn said(kind: Kind, body: &str) -> Option<String> {
    let answer: serde_json::Value = serde_json::from_str(body).ok()?;
    let said = match kind {
        Kind::Remote => answer
            .get("choices")?
            .as_array()?
            .first()?
            .get("message")?
            .get("content")?,
        _ => answer.get("message")?.get("content")?,
    };
    let said = said.as_str()?.trim();
    match said.is_empty() {
        true => None,
        false => Some(said.to_owned()),
    }
}
