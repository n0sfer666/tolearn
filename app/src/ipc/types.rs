use super::dto::dto;
use super::shape::Shape;

dto!(Span { min: u32, max: u32 });

dto!(SettingsIn { save: Option<SettingsView> });
dto!(SettingsView {
    disk_budget_mb: u32,
    locale: String,
    theme: String,
});

dto!(SearchIn {
    query: String,
    limit: u32,
});
dto!(SearchOut {
    hits: Vec<HitView>,
    indexed: u32,
});
dto!(HitView {
    kind: String,
    program: String,
    node: String,
    node_title: String,
    stage: String,
    title: String,
    block: String,
    snippet: String,
});

dto!(LlmLogIn {
    open: bool,
    clear: bool,
});
dto!(LlmLogOut {
    room: String,
    records: u32,
});

dto!(SpeechStateIn { program: String });
dto!(SpeechStateOut {
    available: bool,
    listening: bool,
    language: String,
});
dto!(SpeechStopIn { program: String });
dto!(SpeechStopOut { text: String });

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
    outdated: Option<DriftView>,
});
dto!(DriftView {
    removed: Vec<String>,
    added: Vec<String>,
    fingerprint: String,
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
    journal: bool,
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
    dismissed_advice: Option<String>,
});
dto!(PresetView {
    id: String,
    command: String,
    args: Vec<String>,
    available: bool,
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

pub fn shapes() -> Vec<Shape> {
    let mut shapes = vec![
        Span::shape(),
        SettingsIn::shape(),
        SettingsView::shape(),
        SearchIn::shape(),
        SearchOut::shape(),
        HitView::shape(),
        ProviderIn::shape(),
        ProviderOut::shape(),
        DriftView::shape(),
        ProviderView::shape(),
        HttpView::shape(),
        HarnessView::shape(),
        PresetView::shape(),
        AdviceView::shape(),
        CheckedView::shape(),
        ProbedView::shape(),
        LlmLogIn::shape(),
        LlmLogOut::shape(),
        SpeechStateIn::shape(),
        SpeechStateOut::shape(),
        SpeechStopIn::shape(),
        SpeechStopOut::shape(),
    ];
    shapes.extend(super::reading::shapes());
    shapes.extend(super::planned::shapes());
    shapes
}
