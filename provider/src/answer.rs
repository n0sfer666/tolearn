use crate::ask::{Length, Said};
use crate::types::Api;

pub(crate) fn said(api: Api, body: &str, length: Length) -> Option<Said> {
    let answer: serde_json::Value = serde_json::from_str(body).ok()?;
    let spent = tokens(api, &answer);
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
    if let Some(text) = told("content") {
        return Some(Said {
            text,
            thinking: false,
            tokens: spent,
        });
    }
    match length {
        Length::Full => None,
        Length::Brief => told(thinking(api)).map(|text| Said {
            text,
            thinking: true,
            tokens: spent,
        }),
    }
}

fn tokens(api: Api, answer: &serde_json::Value) -> Option<u32> {
    let counted = match api {
        Api::Ollama => {
            answer.get("eval_count")?.as_u64()?
                + answer
                    .get("prompt_eval_count")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or_default()
        }
        Api::OpenAi => answer.get("usage")?.get("total_tokens")?.as_u64()?,
    };
    u32::try_from(counted).ok()
}

fn thinking(api: Api) -> &'static str {
    match api {
        Api::Ollama => "thinking",
        Api::OpenAi => "reasoning_content",
    }
}
