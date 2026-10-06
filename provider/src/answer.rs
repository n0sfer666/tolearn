use serde_json::Value;

use crate::ask::{Length, Said};
use crate::tokens::Tokens;
use crate::types::Api;

pub(crate) fn said(api: Api, body: &str, length: Length, asked: &str) -> Option<Said> {
    let answer: Value = serde_json::from_str(body).ok()?;
    let message = match api {
        Api::Ollama => answer.get("message")?,
        Api::OpenAi => answer.get("choices")?.as_array()?.first()?.get("message")?,
    };
    let told = |field: &str| {
        message
            .get(field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|said| !said.is_empty())
            .map(str::to_owned)
    };
    let (text, thought) = match (told("content"), length) {
        (Some(text), _) => (text, false),
        (None, Length::Full) => return None,
        (None, Length::Brief) => (told(thinking(api))?, true),
    };
    let model = [
        answer
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        asked,
    ]
    .into_iter()
    .map(str::trim)
    .find(|model| !model.is_empty());
    Some(Said {
        text,
        thinking: thought,
        tokens: tokens(api, &answer),
        model: model.map(str::to_owned),
    })
}

fn tokens(api: Api, answer: &Value) -> Tokens {
    match api {
        Api::Ollama => Tokens {
            input: count(Some(answer), "prompt_eval_count"),
            output: count(Some(answer), "eval_count"),
        },
        Api::OpenAi => {
            let usage = answer.get("usage");
            Tokens {
                input: count(usage, "prompt_tokens"),
                output: count(usage, "completion_tokens"),
            }
        }
    }
}

fn count(holder: Option<&Value>, field: &str) -> Option<u32> {
    let counted = holder?.get(field)?.as_u64()?;
    u32::try_from(counted).ok()
}

fn thinking(api: Api) -> &'static str {
    match api {
        Api::Ollama => "thinking",
        Api::OpenAi => "reasoning_content",
    }
}
