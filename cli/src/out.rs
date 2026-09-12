use serde_json::Value;

#[derive(Debug)]
pub struct Output {
    pub text: String,
    pub json: Value,
}

impl Output {
    pub fn new(text: String, json: Value) -> Self {
        Self { text, json }
    }

    pub fn shown(&self, as_json: bool) -> String {
        if as_json {
            return format!("{:#}", self.json);
        }
        self.text.clone()
    }
}
