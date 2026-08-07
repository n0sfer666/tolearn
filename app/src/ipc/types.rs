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
    title: String,
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
    program: String,
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
    unload: UnloadView,
    practice: PracticeView,
    questions: Vec<QuestionView>,
    exam: ExamView,
});
dto!(UnloadView {
    state: String,
    checked_at: Option<i64>,
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

dto!(PracticeIn {
    bundle: String,
    topic: String,
    step: String,
    now: String,
});
dto!(PracticeOut {
    spent_sec: u32,
    left_sec: i64,
    box_min: u32,
    running: bool,
    expired: bool,
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

dto!(PlanIn {
    bundle: String,
    today: String,
});
dto!(PlanOut {
    weekly_hours: u32,
    daily_hours: f64,
    left: Span,
    unknown: u32,
    soonest: AheadView,
    latest: AheadView,
});
dto!(AheadView {
    days: u32,
    date: String
});

dto!(ExportIn {
    bundle: String,
    today: String,
    path: String,
    directory: Option<String>,
});
dto!(ExportOut {
    path: String,
    bytes: u64,
    plaintext: bool,
});

dto!(EncryptionIn {
    enable: Option<bool>,
    phrase: Option<String>,
});
dto!(EncryptionOut {
    enabled: bool,
    external: bool,
});

dto!(FollowIn { url: String });
dto!(FollowOut {
    program: String,
    topic: String,
});

dto!(StaleIn {
    bundle: String,
    today: String,
});
dto!(StaleOut {
    topics: Vec<ExpiredView>,
    materials: Vec<AgingView>,
});
dto!(ExpiredView {
    topic: String,
    title: String,
    verified_at: String,
    expired_at: String,
});
dto!(AgingView {
    topic: String,
    topic_title: String,
    title: String,
    url: String,
    stale: bool,
    delta: Option<String>,
    covers_version: Option<String>,
    pin: String,
});

dto!(GraphIn {
    bundle: String,
    today: String,
});
dto!(GraphOut {
    nodes: Vec<NodeView>,
});
dto!(NodeView {
    id: String,
    title: String,
    status: String,
    layer: u32,
    depends_on: Vec<String>,
    blocked_by: Vec<String>,
    unlocks: Vec<String>,
});
dto!(StatsIn { bundle: String });
dto!(StatsOut {
    attempts: u32,
    enough: bool,
    hinted: u32,
    hinted_share: f64,
    kinds: Vec<KindView>,
    actions: Vec<ActionView>,
    streak: StreakView,
    calibration: Vec<String>,
});
dto!(KindView {
    kind: String,
    ok: u32,
    partial: u32,
    miss: u32,
});
dto!(ActionView {
    action: String,
    count: u32,
});
dto!(StreakView {
    longest: u32,
    topic: String,
});

dto!(QueueIn { today: String });
dto!(QueueOut { due: Vec<DueView> });
dto!(DueView {
    program: String,
    title: String,
    bundle: String,
    topic: String,
    topic_title: String,
    due: String,
    overdue: bool,
});

dto!(RepeatIn {
    bundle: String,
    topic: String,
    today: String,
});
dto!(RepeatOut { next_review_at: Option<String> });

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

dto!(HistoryIn { bundle: String });
dto!(HistoryOut {
    versions: Vec<VersionView>,
});
dto!(VersionView {
    n: u32,
    saved_at: String,
    bytes: u64,
});
dto!(HistoryDiffIn {
    bundle: String,
    version: u32,
});
dto!(HistoryDiffOut {
    added: Vec<Link>,
    removed: Vec<Link>,
    rewritten: Vec<RewrittenView>,
});
dto!(RewrittenView {
    id: String,
    title: String,
    changed: Vec<String>,
    demoted: bool,
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

dto!(ExamineIn {
    bundle: String,
    topic: String,
});
dto!(ExamineOut { text: String });

dto!(ExamLineView {
    side: String,
    text: String
});
dto!(ExamStateOut {
    open: bool,
    stale: bool,
    stage: String,
    asked: u32,
    total: u32,
    hint_ready: bool,
    log: Vec<ExamLineView>,
    graded: Vec<AnswerView>,
    hinted: Vec<String>,
    seconds: u32,
    tokens: u32,
    verdict: Option<String>
});
dto!(ExamStateIn {
    bundle: String,
    topic: String
});
dto!(ExamStartIn {
    bundle: String,
    topic: String,
    restart: bool
});
dto!(ExamSayIn {
    bundle: String,
    topic: String,
    text: String
});
dto!(ExamFinishIn {
    bundle: String,
    topic: String,
    today: String
});

dto!(SettingsIn { save: Option<SettingsView> });
dto!(SettingsView {
    disk_budget_mb: u32,
    notes_directory: Option<String>,
    locale: String,
    theme: String,
    history_depth: u32,
    history_share_percent: u32,
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

dto!(ProviderIn {
    save: Option<ProviderView>,
    key: Option<String>,
    forget: bool,
    check: bool,
    probe: bool,
});
dto!(ProviderOut {
    provider: ProviderView,
    has_key: bool,
    checked: Option<CheckedView>,
    probed: Option<ProbedView>,
    presets: Vec<PresetView>,
    advised: Vec<AdviceView>,
});
dto!(AdviceView {
    id: String,
    repo: String,
    gigabytes: u32,
    heavy: bool,
    installed: bool,
});
dto!(ProviderView {
    enabled: bool,
    active: String,
    local: HttpView,
    remote: HttpView,
    harness: HarnessView,
});
dto!(HttpView {
    endpoint: String,
    api: String,
    model: String,
    num_ctx: u32,
    temperature_tenths: u32,
});
dto!(HarnessView {
    id: String,
    command: String,
    args: Vec<String>,
    timeout_secs: u32,
});
dto!(PresetView {
    id: String,
    command: String,
    args: Vec<String>,
});
dto!(CheckedView {
    models: Vec<String>,
    version: Option<String>,
    took_ms: Option<u32>,
});
dto!(ProbedView {
    said: String,
    took_ms: u32,
    thinking: bool,
});

dto!(SaveOfflineIn {
    bundle: String,
    topic: Option<String>,
    again: Option<String>,
});
dto!(SaveOfflineOut {
    job: String,
    total: u32
});
dto!(OfflineStateIn { job: String });
dto!(OfflineStateOut {
    total: u32,
    done: u32,
    current: String,
    finished: bool,
    cancelled: bool,
    bytes: u64,
    saved: Vec<String>,
    skipped: Vec<LeftView>,
    failed: Vec<LeftView>,
});
dto!(LeftView {
    url: String,
    why: String
});
dto!(StopOfflineIn { job: String });
dto!(StopOfflineOut { stopping: bool });
dto!(ReadOfflineIn { url: String });
dto!(ReadOfflineOut {
    kind: String,
    title: String,
    html: String,
    text: String,
    blocks: Vec<Block>,
    path: String,
    extracted: bool,
});
dto!(Block {
    kind: String,
    level: u32,
    text: String,
    src: String,
});

dto!(GenerateIn {
    subject: String,
    level: String,
    weekly_hours: u32,
    weeks: Option<u32>,
});
dto!(GenerateOut { job: String });
dto!(GenerateStateIn { job: String });
dto!(GenerateStateOut {
    step: String,
    total: u32,
    done: u32,
    attempt: u32,
    rounds: u32,
    retry: u32,
    tries: u32,
    current: String,
    waiting: bool,
    finished: bool,
    cancelled: bool,
    refused: Vec<String>,
    seconds: u64,
    step_seconds: u64,
    chars: u64,
    ticks: u64,
    tail: String,
    tokens: Option<u32>,
    summary: Option<SummaryView>,
});
dto!(SummaryView {
    id: String,
    title: String,
    topics: u32,
    hours_min: u32,
    hours_max: u32,
    stages: Vec<StagedView>,
});
dto!(StagedView {
    n: u32,
    title: String,
    topics: u32,
    first: Vec<String>,
});
dto!(GenerateGoIn { job: String });
dto!(GenerateGoOut { going: bool });
dto!(GenerateStopIn { job: String });
dto!(GenerateStopOut { stopping: bool });
dto!(GenerateAcceptIn {
    job: String,
    today: String,
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
        UnloadView::shape(),
        MaterialView::shape(),
        PracticeView::shape(),
        CheckView::shape(),
        QuestionView::shape(),
        ExamView::shape(),
        RunCheckIn::shape(),
        RunCheckOut::shape(),
        SetStatusIn::shape(),
        SetStatusOut::shape(),
        PracticeIn::shape(),
        PracticeOut::shape(),
        ProgramsIn::shape(),
        ProgramsOut::shape(),
        Card::shape(),
        PlanIn::shape(),
        PlanOut::shape(),
        AheadView::shape(),
        ExportIn::shape(),
        ExportOut::shape(),
        StaleIn::shape(),
        StaleOut::shape(),
        ExpiredView::shape(),
        AgingView::shape(),
        GraphIn::shape(),
        GraphOut::shape(),
        NodeView::shape(),
        StatsIn::shape(),
        StatsOut::shape(),
        KindView::shape(),
        ActionView::shape(),
        StreakView::shape(),
        QueueIn::shape(),
        QueueOut::shape(),
        DueView::shape(),
        RepeatIn::shape(),
        RepeatOut::shape(),
        ImportIn::shape(),
        ImportOut::shape(),
        Merged::shape(),
        StaleTopic::shape(),
        HistoryIn::shape(),
        HistoryOut::shape(),
        VersionView::shape(),
        HistoryDiffIn::shape(),
        HistoryDiffOut::shape(),
        RewrittenView::shape(),
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
        ExamineIn::shape(),
        ExamineOut::shape(),
        ExamLineView::shape(),
        ExamStateOut::shape(),
        ExamStateIn::shape(),
        ExamStartIn::shape(),
        ExamSayIn::shape(),
        ExamFinishIn::shape(),
        SettingsIn::shape(),
        SettingsView::shape(),
        SearchIn::shape(),
        SearchOut::shape(),
        HitView::shape(),
        EncryptionIn::shape(),
        EncryptionOut::shape(),
        FollowIn::shape(),
        FollowOut::shape(),
        ProviderIn::shape(),
        ProviderOut::shape(),
        ProviderView::shape(),
        HttpView::shape(),
        HarnessView::shape(),
        PresetView::shape(),
        AdviceView::shape(),
        CheckedView::shape(),
        ProbedView::shape(),
        SaveOfflineIn::shape(),
        SaveOfflineOut::shape(),
        OfflineStateIn::shape(),
        OfflineStateOut::shape(),
        LeftView::shape(),
        StopOfflineIn::shape(),
        StopOfflineOut::shape(),
        ReadOfflineIn::shape(),
        ReadOfflineOut::shape(),
        Block::shape(),
        GenerateIn::shape(),
        GenerateOut::shape(),
        GenerateStateIn::shape(),
        GenerateStateOut::shape(),
        SummaryView::shape(),
        StagedView::shape(),
        GenerateGoIn::shape(),
        GenerateGoOut::shape(),
        GenerateStopIn::shape(),
        GenerateStopOut::shape(),
        GenerateAcceptIn::shape(),
    ]
}
