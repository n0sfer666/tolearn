use serde_json::Value;

use crate::ask::Said;

const USAGE: [&str; 3] = [
    "input_tokens",
    "output_tokens",
    "cache_creation_input_tokens",
];

#[derive(Debug, Default)]
pub struct Tape {
    pending: String,
    answer: String,
    tokens: Option<u32>,
    json: bool,
}

impl Tape {
    pub fn feed(&mut self, chunk: &str) -> String {
        self.pending.push_str(chunk);
        let mut shown = String::new();
        while let Some(end) = self.pending.find('\n') {
            let line: String = self.pending.drain(..=end).collect();
            self.line(line.trim(), &mut shown);
        }
        shown
    }

    pub fn heard(&self) -> Option<Said> {
        self.json.then(|| Said {
            text: self.answer.trim().to_owned(),
            thinking: false,
            tokens: self.tokens,
        })
    }

    fn line(&mut self, line: &str, shown: &mut String) {
        if line.is_empty() {
            return;
        }
        let Some(event) = event(line) else {
            if !self.json {
                shown.push_str(line);
                shown.push('\n');
            }
            return;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("stream_event") => self.delta(event.get("event"), shown),
            Some("assistant") => {
                let message = event.get("message");
                self.spent(message);
                self.whole(&blocks(message), shown);
            }
            Some("result") => {
                self.spent(Some(&event));
                self.whole(&text(event.get("result")), shown);
            }
            _ => return,
        }
        self.json = true;
    }

    fn delta(&mut self, event: Option<&Value>, shown: &mut String) {
        let Some(delta) = event.and_then(|event| event.get("delta")) else {
            return;
        };
        match delta.get("type").and_then(Value::as_str) {
            Some("text_delta") => {
                let said = text(delta.get("text"));
                self.answer.push_str(&said);
                shown.push_str(&said);
            }
            Some("thinking_delta") => shown.push_str(&text(delta.get("thinking"))),
            _ => {}
        }
    }

    fn whole(&mut self, said: &str, shown: &mut String) {
        let said = said.trim();
        if said.is_empty() || self.answer.contains(said) {
            return;
        }
        self.answer.push_str(said);
        shown.push_str(said);
    }

    fn spent(&mut self, holder: Option<&Value>) {
        let Some(usage) = holder.and_then(|holder| holder.get("usage")) else {
            return;
        };
        let counted: u64 = USAGE
            .iter()
            .filter_map(|field| usage.get(field))
            .filter_map(Value::as_u64)
            .sum();
        if counted > 0 {
            self.tokens = u32::try_from(counted).ok();
        }
    }
}

fn event(line: &str) -> Option<Value> {
    let event: Value = serde_json::from_str(line).ok()?;
    event.is_object().then_some(event)
}

fn blocks(message: Option<&Value>) -> String {
    message
        .and_then(|message| message.get("content"))
        .and_then(Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|block| block.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .concat()
        })
        .unwrap_or_default()
}

fn text(field: Option<&Value>) -> String {
    field
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_default()
}
