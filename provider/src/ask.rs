use std::time::Duration;

use serde_json::json;

use crate::error::CheckError;
use crate::harness;
use crate::types::{Api, Http, Kind, Provider};
use crate::wire::{apart, broken, client, given, refused};

pub const PATIENCE: Duration = Duration::from_secs(180);

const BRIEF_TOKENS: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Length {
    Full,
    Brief,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Told {
    pub said: String,
    pub thinking: bool,
}

pub fn ask(provider: &Provider, key: Option<&str>, prompt: &str) -> Result<String, CheckError> {
    told(provider, key, prompt, Length::Full).map(|told| told.said)
}

pub(crate) fn briefly(
    provider: &Provider,
    key: Option<&str>,
    prompt: &str,
) -> Result<Told, CheckError> {
    told(provider, key, prompt, Length::Brief)
}

fn told(
    provider: &Provider,
    key: Option<&str>,
    prompt: &str,
    length: Length,
) -> Result<Told, CheckError> {
    if !provider.enabled {
        return Err(CheckError::Disabled);
    }
    match provider.active {
        Kind::Harness => harness::ask(&provider.harness, prompt).map(plainly),
        Kind::Local => spoken(&provider.local, None, prompt, length),
        Kind::Remote => {
            let key = given(key).ok_or(CheckError::NoKey)?;
            spoken(&provider.remote, Some(key), prompt, length)
        }
    }
}

fn plainly(said: String) -> Told {
    Told {
        said,
        thinking: false,
    }
}

fn spoken(
    http: &Http,
    key: Option<&str>,
    prompt: &str,
    length: Length,
) -> Result<Told, CheckError> {
    if http.model.trim().is_empty() {
        return Err(CheckError::NoModel);
    }
    let asked = http.clone();
    let key = key.map(str::to_owned);
    let prompt = prompt.to_owned();
    apart(move || send(&asked, key.as_deref(), &prompt, length))
}

fn send(http: &Http, key: Option<&str>, prompt: &str, length: Length) -> Result<Told, CheckError> {
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
    said(http.api, &body, length).ok_or(CheckError::BadAnswer)
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

fn said(api: Api, body: &str, length: Length) -> Option<Told> {
    let answer: serde_json::Value = serde_json::from_str(body).ok()?;
    let message = match api {
        Api::Ollama => answer.get("message")?,
        Api::OpenAi => answer.get("choices")?.as_array()?.first()?.get("message")?,
    };
    let told = |field: &str| {
        message
            .get(field)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|said| !said.is_empty())
            .map(str::to_owned)
    };
    if let Some(said) = told("content") {
        return Some(plainly(said));
    }
    match length {
        Length::Full => None,
        Length::Brief => told(thinking(api)).map(|said| Told {
            said,
            thinking: true,
        }),
    }
}

fn thinking(api: Api) -> &'static str {
    match api {
        Api::Ollama => "thinking",
        Api::OpenAi => "reasoning_content",
    }
}
