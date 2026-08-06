const ROADMAP: &str = include_str!("../../assets/generate-roadmap.md");
const TOPIC: &str = include_str!("../../assets/generate-topic.md");

pub fn roadmap() -> &'static str {
    ROADMAP
}

pub fn topic() -> &'static str {
    TOPIC
}
