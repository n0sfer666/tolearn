use serde_json::{Value, json};
use tolearn_offline::reach::Reach;

#[derive(Debug)]
pub struct Net(pub bool);

impl Reach for Net {
    fn reach(&self, _url: &str) -> Result<(), String> {
        match self.0 {
            true => Ok(()),
            false => Err("network is unreachable".to_owned()),
        }
    }
}

pub fn provider(endpoint: &str) -> Value {
    let http = |api| {
        json!({
            "endpoint": endpoint,
            "api": api,
            "model": "llama3:8b",
            "num_ctx": 0,
            "temperature_tenths": 7,
        })
    };
    json!({
        "enabled": true,
        "active": "local",
        "journal": false,
        "local": http("ollama"),
        "remote": http("openai"),
        "harness": { "id": "custom", "command": "claude", "args": ["-p"], "timeout_secs": 180 },
    })
}
