mod split;
mod types;

pub use types::{Past, Review, Reviewed};

use crate::progress::{Answer, Attempt, TopicState};
use crate::protocol::{ENOUGH_FAILURES, counted, failing_streak};
use crate::topic::Topic;

pub fn review(topic: &Topic, state: Option<&TopicState>) -> Review {
    let attempts = state.map(|state| state.attempts.as_slice()).unwrap_or(&[]);
    let last = attempts.last();
    let questions = questions(topic, last);
    let streak = failing_streak(attempts);
    let suggested = streak >= ENOUGH_FAILURES;

    Review {
        split_request: if suggested {
            split::request(topic, &questions, &failures(attempts, streak))
        } else {
            String::new()
        },
        loose: loose(&questions, last),
        last: last.map(past),
        history: attempts.iter().rev().skip(1).map(past).collect(),
        split_suggested: suggested,
        questions,
    }
}

fn questions(topic: &Topic, last: Option<&Attempt>) -> Vec<Reviewed> {
    topic
        .questions
        .iter()
        .map(|question| {
            let answer = last.and_then(|attempt| answer(attempt, &question.id));
            Reviewed {
                id: question.id.clone(),
                kind: question.kind.label().to_owned(),
                text: question.text.clone(),
                outcome: answer.map(|answer| answer.outcome),
                missed: answer
                    .map(|answer| answer.missed.clone())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn answer<'a>(attempt: &'a Attempt, id: &str) -> Option<&'a Answer> {
    attempt.per_question.iter().find(|answer| answer.id == id)
}

fn loose(questions: &[Reviewed], last: Option<&Attempt>) -> Vec<String> {
    let attached = |gap: &String| {
        questions
            .iter()
            .any(|question| question.missed.contains(gap))
    };
    last.map(|attempt| {
        attempt
            .gaps
            .iter()
            .filter(|gap| !attached(gap))
            .cloned()
            .collect()
    })
    .unwrap_or_default()
}

fn failures(attempts: &[Attempt], streak: usize) -> Vec<&Attempt> {
    let counted = counted(attempts);
    counted[counted.len() - streak..].to_vec()
}

fn past(attempt: &Attempt) -> Past {
    Past {
        at: attempt.at.clone(),
        verdict: attempt.verdict,
    }
}
