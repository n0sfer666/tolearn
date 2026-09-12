use std::cell::RefCell;

use tolearn_generate::Model;
use tolearn_generate::plan::Request;
use tolearn_offline::reach::Reach;
use tolearn_provider::{CheckError, Said, Tokens};

#[derive(Debug)]
pub struct Up;

impl Reach for Up {
    fn reach(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Scripted {
    answers: RefCell<Vec<String>>,
    prompts: RefCell<Vec<String>>,
}

impl Scripted {
    pub fn new(mut answers: Vec<String>) -> Self {
        answers.reverse();
        Self {
            answers: RefCell::new(answers),
            prompts: RefCell::new(Vec::new()),
        }
    }

    pub fn prompts(&self) -> Vec<String> {
        self.prompts.borrow().clone()
    }
}

impl Model for Scripted {
    fn ask(&self, prompt: &str) -> Result<Said, CheckError> {
        self.prompts.borrow_mut().push(prompt.to_owned());
        let mut answers = self.answers.borrow_mut();
        let text = if answers.len() > 1 {
            answers.pop().unwrap()
        } else {
            answers[0].clone()
        };
        Ok(Said {
            text,
            thinking: false,
            tokens: Tokens::default(),
            model: None,
        })
    }
}

pub fn answer(name: &str) -> String {
    if name.ends_with(".txt") {
        std::fs::read_to_string(format!("../fixtures/generate/plan/{name}")).unwrap()
    } else {
        name.to_owned()
    }
}

pub fn model(answers: &[&str]) -> Scripted {
    Scripted::new(answers.iter().map(|name| answer(name)).collect())
}

pub fn request() -> Request {
    Request {
        request: "Хочу писать чиптюн".to_owned(),
        level: "Нот не знаю, трекер не открывал".to_owned(),
        locale: "ru".to_owned(),
    }
}
