pub struct Bound {
    pub roadmap: String,
    pub topic: String,
    pub body: String,
}

pub fn parse(text: &str) -> Option<Bound> {
    let rest = text.strip_prefix("---\n")?;
    let (head, body) = rest
        .split_once("\n---\n")
        .or_else(|| rest.split_once("\n---"))?;
    Some(Bound {
        roadmap: field(head, "roadmap")?,
        topic: field(head, "topic")?,
        body: body.strip_prefix('\n').unwrap_or(body).to_owned(),
    })
}

pub fn render(roadmap: &str, topic: &str, body: &str) -> String {
    let text = body.strip_suffix('\n').unwrap_or(body);
    format!("---\ntolearn:\n  roadmap: {roadmap}\n  topic: {topic}\n---\n\n{text}\n")
}

fn field(head: &str, name: &str) -> Option<String> {
    head.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(&format!("{name}:")))
        .map(|value| value.trim().trim_matches('"').to_owned())
        .filter(|value| !value.is_empty())
}
