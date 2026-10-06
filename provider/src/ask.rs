use std::time::Duration;

use serde_json::json;
use tolearn_runner::Stop;

use crate::answer::said;
use crate::error::CheckError;
use crate::harness;
use crate::tokens::Tokens;
use crate::types::{Api, Http, Kind, Provider, Watch};
use crate::wire::{apart, broken, client, given, refused};

pub const PATIENCE: Duration = Duration::from_secs(180);

const BRIEF_TOKENS: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Length {
    Full,
    Brief,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    pub text: String,
    pub thinking: bool,
    pub tokens: Tokens,
    pub model: Option<String>,
}

pub fn ask(provider: &Provider, key: Option<&str>, prompt: &str) -> Result<Said, CheckError> {
    told(provider, key, prompt, Length::Full, None, &Stop::default())
}

pub fn stoppable(
    provider: &Provider,
    key: Option<&str>,
    prompt: &str,
    stop: &Stop,
) -> Result<Said, CheckError> {
    told(provider, key, prompt, Length::Full, None, stop)
}

pub fn watched(
    provider: &Provider,
    key: Option<&str>,
    prompt: &str,
    watch: Watch,
) -> Result<Said, CheckError> {
    told(
        provider,
        key,
        prompt,
        Length::Full,
        Some(watch),
        &Stop::default(),
    )
}

pub(crate) fn briefly(
    provider: &Provider,
    key: Option<&str>,
    prompt: &str,
) -> Result<Said, CheckError> {
    told(provider, key, prompt, Length::Brief, None, &Stop::default())
}

fn told(
    provider: &Provider,
    key: Option<&str>,
    prompt: &str,
    length: Length,
    watch: Option<Watch>,
    stop: &Stop,
) -> Result<Said, CheckError> {
    if !provider.enabled {
        return Err(CheckError::Disabled);
    }
    if stop.stopped() {
        return Err(CheckError::Cancelled);
    }
    match provider.active {
        Kind::Harness => harness::ask(&provider.harness, prompt, watch, stop),
        Kind::Local => spoken(&provider.local, None, prompt, length, stop),
        Kind::Remote => {
            let key = given(key).ok_or(CheckError::NoKey)?;
            spoken(&provider.remote, Some(key), prompt, length, stop)
        }
    }
}

fn spoken(
    http: &Http,
    key: Option<&str>,
    prompt: &str,
    length: Length,
    stop: &Stop,
) -> Result<Said, CheckError> {
    if http.model.trim().is_empty() {
        return Err(CheckError::NoModel);
    }
    let asked = http.clone();
    let key = key.map(str::to_owned);
    let prompt = prompt.to_owned();
    apart(move || send(&asked, key.as_deref(), &prompt, length), stop)
}

fn send(http: &Http, key: Option<&str>, prompt: &str, length: Length) -> Result<Said, CheckError> {
    let mut request = client(PATIENCE)?
        .post(route(http))
        .header("content-type", "application/json")
        .body(body(http, prompt, length).to_string());
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }

    let answer = request.send().map_err(broken)?;
    if let Some(refused) = refused(answer.status().as_u16()) {
        return Err(refused);
    }

    let body = answer.text().map_err(broken)?;
    said(http.api, &body, length, &http.model).ok_or(CheckError::BadAnswer)
}

fn route(http: &Http) -> String {
    let base = http.endpoint.trim_end_matches('/');
    match http.api {
        Api::Ollama => format!("{base}/api/chat"),
        Api::OpenAi => format!("{base}/chat/completions"),
    }
}

fn body(http: &Http, prompt: &str, length: Length) -> serde_json::Value {
    let mut body = json!({
        "model": http.model,
        "messages": [{ "role": "user", "content": prompt }],
        "stream": false,
    });
    let heat = json!(f64::from(http.temperature_tenths) / 10.0);
    match http.api {
        Api::Ollama => body["options"] = options(http, heat, length),
        Api::OpenAi => {
            body["temperature"] = heat;
            if length == Length::Brief {
                body["max_tokens"] = json!(BRIEF_TOKENS);
            }
        }
    }
    body
}

fn options(http: &Http, heat: serde_json::Value, length: Length) -> serde_json::Value {
    let mut options = json!({ "temperature": heat });
    if http.num_ctx > 0 {
        options["num_ctx"] = json!(http.num_ctx);
    }
    if length == Length::Brief {
        options["num_predict"] = json!(BRIEF_TOKENS);
    }
    options
}
