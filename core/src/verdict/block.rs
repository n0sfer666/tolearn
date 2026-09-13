use serde_json::{Deserializer, Map, Value};

pub(super) fn last(text: &str) -> Option<Map<String, Value>> {
    let mut found = None;
    let mut from = 0;
    while let Some(offset) = text[from..].find('{') {
        let start = from + offset;
        let mut values = Deserializer::from_str(&text[start..]).into_iter::<Value>();
        match values.next() {
            Some(Ok(Value::Object(object))) => {
                found = Some(object);
                from = start + values.byte_offset();
            }
            _ => from = start + 1,
        }
    }
    found
}
