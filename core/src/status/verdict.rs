use crate::progress::{Status, Verdict};

pub fn from_verdict(verdict: Verdict) -> Status {
    match verdict {
        Verdict::Pass => Status::Passed,
        Verdict::Partial | Verdict::Blocked => Status::InProgress,
        Verdict::Fail => Status::Failed,
    }
}
