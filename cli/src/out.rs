use serde_json::Value;

#[derive(Debug)]
pub struct Output {
    pub text: String,
    pub json: Value,
    pub ok: bool,
}

impl Output {
    pub fn new(text: String, json: Value) -> Self {
        Self {
            text,
            json,
            ok: true,
        }
    }

    pub fn verdict(text: String, json: Value, ok: bool) -> Self {
        Self { text, json, ok }
    }

    pub fn shown(&self, as_json: bool) -> String {
        if as_json {
            return format!("{:#}", self.json);
        }
        self.text.clone()
    }

    pub fn code(&self) -> i32 {
        if self.ok { 0 } else { 1 }
    }
}
