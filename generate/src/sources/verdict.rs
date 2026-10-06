use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict<T> {
    Passed(T),
    Refused(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome<T> {
    pub checked_at: i64,
    pub verdict: Verdict<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verified {
    pub url: String,
    pub title: Option<String>,
    pub text: String,
}
