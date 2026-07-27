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
    checkpoint: String,
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
    title: String,
    stage: u32,
    checkpoint: bool,
    status: String,
    hours: Span,
    blocked_by: Vec<Link>,
});
dto!(Link {
    id: String,
    title: String
});

dto!(ProgramsIn { today: String });
dto!(ProgramsOut { programs: Vec<Card> });
dto!(Card {
    id: String,
    title: String,
    path: String,
    reachable: bool,
    opened_at: Option<String>,
    tally: Option<Tally>,
});

dto!(ImportIn {
    path: String,
    today: String,
});
dto!(ImportOut {
    ok: bool,
    id: Option<String>,
    title: Option<String>,
    violations: Vec<Violation>,
    report: Option<Merged>,
});
dto!(Merged {
    kept: Vec<String>,
    added: Vec<String>,
    orphaned: Vec<String>,
    stale: Vec<StaleTopic>,
});
dto!(StaleTopic {
    id: String,
    changed: Vec<String>,
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
        Link::shape(),
        ProgramsIn::shape(),
        ProgramsOut::shape(),
        Card::shape(),
        ImportIn::shape(),
        ImportOut::shape(),
        Merged::shape(),
        StaleTopic::shape(),
        PromptIn::shape(),
        PromptOut::shape(),
    ]
}
