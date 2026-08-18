pub const EXTENSION: &str = "md";

pub enum Answer<'a> {
    Said(&'a str),
    Refused(&'a str),
}

pub fn text(kind: &str, prompt: &str, answer: &Answer<'_>) -> String {
    let (heading, body) = match answer {
        Answer::Said(said) => ("Ответ", *said),
        Answer::Refused(refusal) => ("Отказ", *refusal),
    };
    format!("# {kind}\n\n## Запрос\n\n{prompt}\n\n## {heading}\n\n{body}\n")
}

pub fn name(seconds: u64, turn: usize) -> String {
    format!("{seconds:010}-{turn:04}.{EXTENSION}")
}
