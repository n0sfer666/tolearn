const TURN: &str = include_str!("../../assets/exam-turn.md");
const HINT: &str = include_str!("../../assets/exam-hint.md");
const PRACTICE: &str = include_str!("../../assets/exam-practice.md");

pub fn turn() -> &'static str {
    TURN
}

pub fn hint() -> &'static str {
    HINT
}

pub fn practice() -> &'static str {
    PRACTICE
}
