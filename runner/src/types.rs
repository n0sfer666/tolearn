use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub timeout: Duration,
    pub output_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Finished { code: Option<i32> },
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub outcome: Outcome,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
}
