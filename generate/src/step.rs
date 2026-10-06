#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Plan,
    Revise,
    Fork,
    Part,
    Sources,
    Text,
    Repair(usize),
    Diagrams,
    Write,
    Exam,
    Clarify,
}

impl Step {
    pub fn label(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Revise => "revise",
            Self::Fork => "fork",
            Self::Part => "part",
            Self::Sources => "sources",
            Self::Text => "text",
            Self::Repair(_) => "repair",
            Self::Diagrams => "diagrams",
            Self::Write => "write",
            Self::Exam => "exam",
            Self::Clarify => "clarify",
        }
    }
}
