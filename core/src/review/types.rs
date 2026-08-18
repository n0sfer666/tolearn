use crate::progress::{Outcome, Verdict};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub questions: Vec<Reviewed>,
    pub loose: Vec<String>,
    pub last: Option<Past>,
    pub history: Vec<Past>,
    pub split_suggested: bool,
    pub split_request: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reviewed {
    pub id: String,
    pub kind: String,
    pub text: String,
    pub outcome: Option<Outcome>,
    pub missed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Past {
    pub at: String,
    pub verdict: Verdict,
}
