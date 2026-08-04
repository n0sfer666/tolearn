use std::time::Duration;

use crate::error::CheckError;
use crate::harness;
use crate::types::{Api, Http, Kind, Provider};
use crate::wire::{apart, broken, client, given, refused};

pub const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    pub models: Vec<String>,
    pub version: Option<String>,
}

pub fn check(provider: &Provider, key: Option<&str>) -> Result<Checked, CheckError> {
    if !provider.enabled {
        return Err(CheckError::Disabled);
    }
    match provider.active {
        Kind::Harness => harness::version(&provider.harness).map(spoke),
        Kind::Local => listed(&provider.local, None),
        Kind::Remote => {
            let key = given(key).ok_or(CheckError::NoKey)?;
            listed(&provider.remote, Some(key))
        }
    }
}

fn spoke(version: String) -> Checked {
    Checked {
        models: Vec::new(),
        version: Some(version),
    }
}

fn listed(http: &Http, key: Option<&str>) -> Result<Checked, CheckError> {
    let asked = http.clone();
    let key = key.map(str::to_owned);
    apart(move || fetch(&asked, key.as_deref()))
}

fn fetch(http: &Http, key: Option<&str>) -> Result<Checked, CheckError> {
    let mut request = client(TIMEOUT)?.get(route(http));
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }

    let answer = request.send().map_err(broken)?;
    if let Some(refused) = refused(answer.status().as_u16()) {
        return Err(refused);
    }

    let body = answer.text().map_err(broken)?;
    models(http.api, &body)
        .map(|models| Checked {
            models,
            version: None,
        })
        .ok_or(CheckError::BadAnswer)
}

fn route(http: &Http) -> String {
    let base = http.endpoint.trim_end_matches('/');
    match http.api {
        Api::Ollama => format!("{base}/api/tags"),
        Api::OpenAi => format!("{base}/models"),
    }
}

fn models(api: Api, body: &str) -> Option<Vec<String>> {
    let (list, name) = match api {
        Api::Ollama => ("models", "name"),
        Api::OpenAi => ("data", "id"),
    };
    let answer: serde_json::Value = serde_json::from_str(body).ok()?;
    let found = answer
        .get(list)?
        .as_array()?
        .iter()
        .filter_map(|item| Some(item.get(name)?.as_str()?.to_owned()))
        .collect();
    Some(found)
}
