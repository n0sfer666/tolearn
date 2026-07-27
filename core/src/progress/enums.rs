#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Todo,
    InProgress,
    ExamPending,
    Passed,
    PassedOut,
    StalePassed,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Exam,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Partial,
    Fail,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Partial,
    Miss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextAction {
    Proceed,
    RetryFailed,
    RedoPractice,
    SplitTopic,
}

impl Status {
    pub fn label(self) -> &'static str {
        STATUS
            .iter()
            .find(|(_, status)| *status == self)
            .map_or("todo", |(label, _)| label)
    }
}

impl Source {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Exam => "exam",
            Self::Manual => "manual",
        }
    }
}

impl Verdict {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Partial => "partial",
            Self::Fail => "fail",
            Self::Blocked => "blocked",
        }
    }
}

impl Outcome {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Partial => "partial",
            Self::Miss => "miss",
        }
    }
}

impl NextAction {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Proceed => "proceed",
            Self::RetryFailed => "retry_failed",
            Self::RedoPractice => "redo_practice",
            Self::SplitTopic => "split_topic",
        }
    }
}

pub(super) const STATUS: [(&str, Status); 8] = [
    ("todo", Status::Todo),
    ("in_progress", Status::InProgress),
    ("exam_pending", Status::ExamPending),
    ("passed", Status::Passed),
    ("passed_out", Status::PassedOut),
    ("stale_passed", Status::StalePassed),
    ("blocked", Status::Blocked),
    ("failed", Status::Failed),
];

pub(super) const SOURCE: [(&str, Source); 2] = [("exam", Source::Exam), ("manual", Source::Manual)];

pub(crate) const VERDICT: [(&str, Verdict); 4] = [
    ("pass", Verdict::Pass),
    ("partial", Verdict::Partial),
    ("fail", Verdict::Fail),
    ("blocked", Verdict::Blocked),
];

pub(crate) const OUTCOME: [(&str, Outcome); 3] = [
    ("ok", Outcome::Ok),
    ("partial", Outcome::Partial),
    ("miss", Outcome::Miss),
];

pub(crate) const NEXT_ACTION: [(&str, NextAction); 4] = [
    ("proceed", NextAction::Proceed),
    ("retry_failed", NextAction::RetryFailed),
    ("redo_practice", NextAction::RedoPractice),
    ("split_topic", NextAction::SplitTopic),
];
