use crate::types::Api;

const HEADROOM: u32 = 4;

#[derive(Debug)]
struct Model {
    id: &'static str,
    repo: &'static str,
    gigabytes: u32,
}

const MODELS: [Model; 6] = [
    Model {
        id: "qwen3:4b",
        repo: "unsloth/Qwen3-4B-GGUF:Q4_K_M",
        gigabytes: 3,
    },
    Model {
        id: "qwen3:8b",
        repo: "unsloth/Qwen3-8B-GGUF:Q4_K_M",
        gigabytes: 5,
    },
    Model {
        id: "gemma3:12b",
        repo: "unsloth/gemma-3-12b-it-GGUF:Q4_K_M",
        gigabytes: 8,
    },
    Model {
        id: "qwen3:14b",
        repo: "unsloth/Qwen3-14B-GGUF:Q4_K_M",
        gigabytes: 9,
    },
    Model {
        id: "gpt-oss:20b",
        repo: "unsloth/gpt-oss-20b-GGUF:Q4_K_M",
        gigabytes: 12,
    },
    Model {
        id: "qwen3:32b",
        repo: "unsloth/Qwen3-32B-GGUF:Q4_K_M",
        gigabytes: 20,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advice {
    pub model: String,
    pub command: String,
    pub gigabytes: u32,
    pub heavy: bool,
}

pub fn advised(api: Api, gigabytes: u32) -> Vec<Advice> {
    MODELS
        .iter()
        .map(|model| Advice {
            model: named(model, api).to_owned(),
            command: command(model, api),
            gigabytes: model.gigabytes,
            heavy: gigabytes > 0 && model.gigabytes + HEADROOM > gigabytes,
        })
        .collect()
}

pub fn known(models: &[String], asked: &str) -> bool {
    let asked = asked.trim().to_lowercase();
    if asked.is_empty() {
        return true;
    }
    models.iter().any(|found| {
        let found = found.trim().to_lowercase();
        !found.is_empty() && (found.contains(&asked) || asked.contains(&found))
    })
}

fn named(model: &Model, api: Api) -> &'static str {
    match api {
        Api::Ollama => model.id,
        Api::OpenAi => model.repo,
    }
}

fn command(model: &Model, api: Api) -> String {
    match api {
        Api::Ollama => format!("ollama pull {}", model.id),
        Api::OpenAi => format!("llama-server -hf {}", model.repo),
    }
}
