use crate::progress::{Attempt, Verdict};

pub const ENOUGH_FAILURES: usize = 3;

pub fn counted(attempts: &[Attempt]) -> Vec<&Attempt> {
    attempts
        .iter()
        .filter(|attempt| attempt.verdict != Verdict::Blocked)
        .collect()
}

pub fn failing_streak(attempts: &[Attempt]) -> usize {
    counted(attempts)
        .iter()
        .rev()
        .take_while(|attempt| attempt.verdict == Verdict::Fail)
        .count()
}
