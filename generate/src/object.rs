pub(crate) fn object(text: &str) -> Result<&str, String> {
    match (text.find('{'), text.rfind('}')) {
        (Some(start), Some(end)) if start < end => Ok(&text[start..=end]),
        _ => Err("в ответе нет объекта JSON".to_owned()),
    }
}
