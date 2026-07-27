use super::dto::dto;
use super::shape::Shape;

dto!(ValidateIn { bundle: String });
dto!(ValidateOut {
    ok: bool,
    violations: Vec<Violation>,
});
dto!(Violation {
    code: String,
    message: String,
});

dto!(ScanIn { bundle: String });
dto!(ScanOut {
    root: String,
    format: String,
    topics: Vec<String>,
    absent: Vec<Absent>,
    broken: Vec<Broken>,
});
dto!(Absent {
    id: String,
    stage: u32,
    generated: bool,
    file: String,
});
dto!(Broken {
    id: String,
    file: String,
    message: String,
});

dto!(ProgramIn {
    bundle: String,
    today: String,
});
dto!(ProgramOut {
    program: Tally,
    stages: Vec<Stage>,
    topics: Vec<TopicStatus>,
});
dto!(Stage {
    n: u32,
    title: String,
    tally: Tally,
});
dto!(Tally {
    done: u32,
    total: u32,
    stale: u32,
    share: f64,
    hours_done: Span,
    hours_total: Span,
});
dto!(Span { min: u32, max: u32 });
dto!(TopicStatus {
    id: String,
    status: String,
});

dto!(PromptIn {
    bundle: String,
    topic: String,
});
dto!(PromptOut { text: String });

pub fn shapes() -> Vec<Shape> {
    vec![
        ValidateIn::shape(),
        ValidateOut::shape(),
        Violation::shape(),
        ScanIn::shape(),
        ScanOut::shape(),
        Absent::shape(),
        Broken::shape(),
        ProgramIn::shape(),
        ProgramOut::shape(),
        Stage::shape(),
        Tally::shape(),
        Span::shape(),
        TopicStatus::shape(),
        PromptIn::shape(),
        PromptOut::shape(),
    ]
}
