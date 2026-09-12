use std::path::PathBuf;

use serde_json::{Value, json};

use super::speaking::{Speaking, speaking};

pub const FORK: &str = r#"{"next": {"why": "Голоса идут сразу за первым звуком", "recommended": false}, "alternatives": [{"id": "noise", "title": "Шумовой канал", "hours": [2, 3], "why": "Ударные без шума не собрать", "recommended": true}]}"#;
const LONG: &str = "Скважность импульса меняет тембр. ";

pub fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("рядом с cli лежит корень репозитория")
        .to_path_buf()
}

fn fixture(path: &str) -> String {
    std::fs::read_to_string(repository().join("fixtures").join(path)).unwrap()
}

fn fine() -> String {
    let text = fixture("generate/stage/stage.txt");
    let mut stage: Value = serde_json::from_str(&text[text.find('{').unwrap()..]).unwrap();
    let blocks = stage["blocks"].as_array_mut().unwrap();
    blocks[3]["text"] = json!("Импульсная волна со скважностью 25%");
    blocks.push(json!({ "kind": "paragraph", "text": LONG.repeat(100), "sources": ["b1"] }));
    stage.to_string()
}

pub fn flat() -> Vec<String> {
    vec![
        fixture("generate/plan/flat.txt"),
        fixture("generate/stage/sources.txt"),
        fine(),
    ]
}

pub fn forking() -> Vec<String> {
    let mut answers = flat();
    answers.push(FORK.to_owned());
    answers.extend(flat().into_iter().skip(1));
    answers
}

pub fn told(answers: Vec<String>) -> Speaking {
    said((7, 11), answers)
}

pub fn said(tokens: (u32, u32), answers: Vec<String>) -> Speaking {
    speaking(tokens, move |_, turn| {
        answers[turn.min(answers.len() - 1)].clone()
    })
}
