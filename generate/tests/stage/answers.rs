use serde_json::{Value, json};
use tolearn_core::block;
use tolearn_core::program::Program;
use tolearn_generate::stage::{self, Draft, Place};
use tolearn_generate::{GenerateError, online};

use crate::support::{Scripted, Up};
use crate::web::{Bench, gathered};

pub const LONG: &str = "Скважность импульса меняет тембр. ";
pub const CALLOUT: &str = "Треугольный канал не меняет громкость";
pub const DELIVERABLE: &str = "Файл voices.ftm";
pub const QUESTION: &str = "Сколько каналов";

pub fn fine() -> Value {
    let text = std::fs::read_to_string("../fixtures/generate/stage/stage.txt").unwrap();
    let mut stage: Value = serde_json::from_str(&text[text.find('{').unwrap()..]).unwrap();
    let blocks = stage["blocks"].as_array_mut().unwrap();
    blocks[3]["text"] = json!("Импульсная волна со скважностью 25%");
    blocks.push(json!({"kind": "paragraph", "text": LONG.repeat(100), "sources": ["b1"]}));
    stage
}

pub fn ids(stage: &Value) -> Vec<String> {
    let texts: Vec<&str> = stage["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .chain(stage["practice"]["task"].as_array().unwrap())
        .map(|block| block["text"].as_str().unwrap())
        .collect();
    block::ids(texts)
}

pub fn drafted(draft: &Draft) -> Vec<String> {
    draft
        .stage
        .every_block()
        .map(|block| block.id.clone())
        .collect()
}

pub struct Run {
    pub result: Result<Draft, GenerateError>,
    pub prompts: Vec<String>,
}

pub fn compose(label: &str, program: &Program, stage: &str, answers: &[Value]) -> Run {
    let (gathered, _) = gathered(&mut Bench::new(label), program, &["sources.txt"]);
    let model = Scripted::new(
        answers
            .iter()
            .map(|answer| match answer {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            })
            .collect(),
    );
    let place = Place::find(program, stage).unwrap();
    let result = stage::compose(&online(&Up, &model).unwrap(), &place, &gathered);
    Run {
        result,
        prompts: model.prompts(),
    }
}
