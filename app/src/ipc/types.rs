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

dto!(TopicIn {
    bundle: String,
    topic: String,
    today: String,
});
dto!(TopicOut {
    id: String,
    title: String,
    stage: u32,
    checkpoint: bool,
    status: String,
    hours: Span,
    blocked_by: Vec<Link>,
    verified_at: String,
    outdated: bool,
    outcomes: Vec<String>,
    misconceptions: Vec<String>,
    materials: Vec<MaterialView>,
    practice: PracticeView,
    questions: Vec<QuestionView>,
    exam: ExamView,
});
dto!(MaterialView {
    title: String,
    url: String,
    kind: String,
    tier: String,
    lang: String,
    stale: bool,
    delta: Option<String>,
    offline: String,
    note: String,
});
dto!(PracticeView {
    kind: String,
    tier: String,
    task: String,
    deliverable: String,
    starting_point: Option<String>,
    fallback: Option<String>,
    time_box_min: u32,
    smoke_checked: bool,
    constraints: Vec<CheckView>,
    acceptance: Vec<CheckView>,
});
dto!(CheckView {
    id: String,
    claim: String,
    check: String,
    expect: String,
});
dto!(QuestionView {
    id: String,
    kind: String,
    text: String,
});
dto!(ExamView {
    focus: String,
    artifact_required: bool,
    max_exchanges: u32,
});

dto!(RunCheckIn {
    bundle: String,
    topic: String,
    check: String
});
dto!(RunCheckOut {
    id: String,
    command: String,
    expect: String,
    code: Option<i32>,
    timed_out: bool,
    stdout: String,
    stderr: String,
    truncated: bool
});

dto!(SetStatusIn {
    bundle: String,
    topic: String,
    status: String,
    today: String,
});
dto!(SetStatusOut { status: String });

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

dto!(StampView {
    modified_nanos: String,
    size: u64
});
dto!(NoteIn {
    bundle: String,
    topic: String,
    directory: Option<String>
});
dto!(NoteOut {
    body: String,
    path: Option<String>,
    stamp: Option<StampView>
});
dto!(SaveNoteIn {
    bundle: String,
    topic: String,
    body: String,
    directory: Option<String>,
    stamp: Option<StampView>
});
dto!(SaveNoteOut {
    saved: bool,
    stamp: Option<StampView>,
    theirs: Option<String>
});

dto!(AnswerView {
    id: String,
    outcome: String,
    missed: Vec<String>
});
dto!(VerdictView {
    result: String,
    status: String,
    gaps: Vec<String>,
    notes: Vec<String>,
    missing: Vec<String>,
    unknown_questions: Vec<String>,
    failed_checks: Vec<String>,
    per_question: Vec<AnswerView>,
    hinted: bool,
    practice_accepted: bool,
    next_action: Option<String>,
    retry_after_days: Option<u32>
});
dto!(ParseVerdictIn {
    bundle: String,
    topic: String,
    text: String
});
dto!(ApplyVerdictIn {
    bundle: String,
    topic: String,
    text: String,
    today: String
});
dto!(ApplyVerdictOut {
    status: String,
    gaps: Vec<String>,
    retry: Vec<String>,
    split_suggested: bool
});

dto!(ReviewedView {
    id: String,
    kind: String,
    text: String,
    outcome: Option<String>,
    missed: Vec<String>,
});
dto!(PastView {
    at: String,
    verdict: String
});
dto!(ReviewIn {
    bundle: String,
    topic: String,
});
dto!(ReviewOut {
    id: String,
    title: String,
    questions: Vec<ReviewedView>,
    loose: Vec<String>,
    last: Option<PastView>,
    history: Vec<PastView>,
    split_suggested: bool,
    split_request: String,
});

dto!(PromptIn {
    bundle: String,
    topic: String,
});
dto!(PromptOut { text: String });

dto!(SettingsIn { save: Option<SettingsView> });
dto!(SettingsView {
    disk_budget_mb: u32,
    notes_directory: Option<String>,
    locale: String,
    theme: String,
});

dto!(SearchIn {
    bundle: String,
    query: String,
    directory: Option<String>,
    limit: u32,
});
dto!(SearchOut {
    hits: Vec<HitView>,
    indexed: u32,
});
dto!(HitView {
    kind: String,
    roadmap: String,
    topic: String,
    title: String,
    snippet: String,
});

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
        TopicIn::shape(),
        TopicOut::shape(),
        MaterialView::shape(),
        PracticeView::shape(),
        CheckView::shape(),
        QuestionView::shape(),
        ExamView::shape(),
        RunCheckIn::shape(),
        RunCheckOut::shape(),
        SetStatusIn::shape(),
        SetStatusOut::shape(),
        ProgramsIn::shape(),
        ProgramsOut::shape(),
        Card::shape(),
        ImportIn::shape(),
        ImportOut::shape(),
        Merged::shape(),
        StaleTopic::shape(),
        StampView::shape(),
        NoteIn::shape(),
        NoteOut::shape(),
        SaveNoteIn::shape(),
        SaveNoteOut::shape(),
        AnswerView::shape(),
        VerdictView::shape(),
        ParseVerdictIn::shape(),
        ApplyVerdictIn::shape(),
        ApplyVerdictOut::shape(),
        ReviewedView::shape(),
        PastView::shape(),
        ReviewIn::shape(),
        ReviewOut::shape(),
        PromptIn::shape(),
        PromptOut::shape(),
        SettingsIn::shape(),
        SettingsView::shape(),
        SearchIn::shape(),
        SearchOut::shape(),
        HitView::shape(),
    ]
}
