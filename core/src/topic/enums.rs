#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Volatility {
    Stable,
    Evolving,
    Volatile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retention {
    ByUse,
    BySchedule,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialType {
    Docs,
    Article,
    Post,
    Guide,
    Spec,
    Rfc,
    Repo,
    Video,
    Paper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialTier {
    T1,
    T2,
    T3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    Ok,
    Paywall,
    LoginRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeKind {
    Code,
    Ops,
    Analysis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeTier {
    P1,
    P2,
    P3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionType {
    Misconception,
    Diagnose,
    Boundary,
    Predict,
    Tradeoff,
}

pub(super) const VOLATILITY: [(&str, Volatility); 3] = [
    ("stable", Volatility::Stable),
    ("evolving", Volatility::Evolving),
    ("volatile", Volatility::Volatile),
];

pub(super) const CONFIDENCE: [(&str, Confidence); 3] = [
    ("high", Confidence::High),
    ("medium", Confidence::Medium),
    ("low", Confidence::Low),
];

pub(super) const RETENTION: [(&str, Retention); 3] = [
    ("by_use", Retention::ByUse),
    ("by_schedule", Retention::BySchedule),
    ("none", Retention::None),
];

pub(super) const MATERIAL_TYPE: [(&str, MaterialType); 9] = [
    ("docs", MaterialType::Docs),
    ("article", MaterialType::Article),
    ("post", MaterialType::Post),
    ("guide", MaterialType::Guide),
    ("spec", MaterialType::Spec),
    ("rfc", MaterialType::Rfc),
    ("repo", MaterialType::Repo),
    ("video", MaterialType::Video),
    ("paper", MaterialType::Paper),
];

pub(super) const MATERIAL_TIER: [(&str, MaterialTier); 3] = [
    ("T1", MaterialTier::T1),
    ("T2", MaterialTier::T2),
    ("T3", MaterialTier::T3),
];

pub(super) const LIVENESS: [(&str, Liveness); 3] = [
    ("ok", Liveness::Ok),
    ("paywall", Liveness::Paywall),
    ("login_required", Liveness::LoginRequired),
];

pub(super) const PRACTICE_KIND: [(&str, PracticeKind); 3] = [
    ("code", PracticeKind::Code),
    ("ops", PracticeKind::Ops),
    ("analysis", PracticeKind::Analysis),
];

pub(super) const PRACTICE_TIER: [(&str, PracticeTier); 3] = [
    ("P1", PracticeTier::P1),
    ("P2", PracticeTier::P2),
    ("P3", PracticeTier::P3),
];

fn label_of<T: PartialEq>(table: &[(&'static str, T)], value: T) -> &'static str {
    table
        .iter()
        .find(|(_, known)| *known == value)
        .map(|(label, _)| *label)
        .unwrap_or("unknown")
}

impl QuestionType {
    pub fn label(self) -> &'static str {
        label_of(&QUESTION_TYPE, self)
    }
}

impl MaterialType {
    pub fn label(self) -> &'static str {
        label_of(&MATERIAL_TYPE, self)
    }
}

impl MaterialTier {
    pub fn label(self) -> &'static str {
        label_of(&MATERIAL_TIER, self)
    }
}

impl Liveness {
    pub fn label(self) -> &'static str {
        label_of(&LIVENESS, self)
    }
}

impl PracticeKind {
    pub fn label(self) -> &'static str {
        label_of(&PRACTICE_KIND, self)
    }
}

impl PracticeTier {
    pub fn label(self) -> &'static str {
        label_of(&PRACTICE_TIER, self)
    }
}

pub(super) const QUESTION_TYPE: [(&str, QuestionType); 5] = [
    ("misconception", QuestionType::Misconception),
    ("diagnose", QuestionType::Diagnose),
    ("boundary", QuestionType::Boundary),
    ("predict", QuestionType::Predict),
    ("tradeoff", QuestionType::Tradeoff),
];
