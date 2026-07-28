use std::time::Duration;

use crate::error::CheckError;
use crate::types::{Flavor, Provider};

pub const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    pub models: Vec<String>,
}

pub fn check(provider: &Provider, key: Option<&str>) -> Result<Checked, CheckError> {
    if !provider.enabled {
        return Err(CheckError::Disabled);
    }
    let key = key.map(str::trim).filter(|key| !key.is_empty());
    if provider.flavor == Flavor::OpenAi && key.is_none() {
        return Err(CheckError::NoKey);
    }

    let asked = provider.clone();
    let key = key.map(str::to_owned);
    std::thread::spawn(move || probe(&asked, key.as_deref()))
        .join()
        .map_err(|_| CheckError::Unreachable("проверка оборвалась".to_owned()))?
}

fn probe(provider: &Provider, key: Option<&str>) -> Result<Checked, CheckError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(unreachable)?;
    let mut request = client.get(route(provider));
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }

    let answer = request.send().map_err(unreachable)?;
    let status = answer.status().as_u16();
    if status == 401 || status == 403 {
        return Err(CheckError::Rejected);
    }
    if !(200..300).contains(&status) {
        return Err(CheckError::Answered(status));
    }

    let body = answer.text().map_err(unreachable)?;
    models(provider.flavor, &body)
        .map(|models| Checked { models })
        .ok_or(CheckError::BadAnswer)
}

fn route(provider: &Provider) -> String {
    let base = provider.endpoint.trim_end_matches('/');
    match provider.flavor {
        Flavor::Ollama => format!("{base}/api/tags"),
        Flavor::OpenAi => format!("{base}/models"),
    }
}

fn models(flavor: Flavor, body: &str) -> Option<Vec<String>> {
    let (list, name) = match flavor {
        Flavor::Ollama => ("models", "name"),
        Flavor::OpenAi => ("data", "id"),
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

fn unreachable(error: reqwest::Error) -> CheckError {
    CheckError::Unreachable(error.to_string())
}
