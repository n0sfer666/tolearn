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
    pub id: String,
    pub repo: String,
    pub gigabytes: u32,
    pub heavy: bool,
}

pub fn advised(gigabytes: u32) -> Vec<Advice> {
    MODELS
        .iter()
        .map(|model| Advice {
            id: model.id.to_owned(),
            repo: model.repo.to_owned(),
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
