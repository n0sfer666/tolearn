mod types;

pub use types::{ActionTally, KindTally, Stats, Streak};

use std::collections::BTreeMap;

use crate::progress::{Attempt, Outcome, Progress, TopicState, Verdict};
use crate::protocol::counted;
use crate::topic::Topic;

pub const ENOUGH: u32 = 5;

pub fn stats(topics: &[Topic], progress: &Progress) -> Stats {
    let attempts: Vec<(&Topic, &Attempt)> = topics
        .iter()
        .flat_map(|topic| {
            progress
                .state(&topic.id)
                .map(|state| state.attempts.iter().map(move |attempt| (topic, attempt)))
                .into_iter()
                .flatten()
        })
        .collect();
    let calibration = attempts
        .iter()
        .rev()
        .filter_map(|(_, attempt)| attempt.calibration.clone())
        .collect();
    let total = u32::try_from(attempts.len()).unwrap_or(u32::MAX);

    if total < ENOUGH {
        return Stats {
            attempts: total,
            calibration,
            ..Stats::default()
        };
    }

    let hinted = attempts
        .iter()
        .filter(|(_, attempt)| attempt.hinted)
        .count();
    let hinted = u32::try_from(hinted).unwrap_or(u32::MAX);

    Stats {
        attempts: total,
        enough: true,
        hinted,
        hinted_share: f64::from(hinted) / f64::from(total),
        kinds: kinds(&attempts),
        actions: actions(&attempts),
        streak: streak(topics, progress),
        calibration,
    }
}

fn kinds(attempts: &[(&Topic, &Attempt)]) -> Vec<KindTally> {
    let mut tallies: BTreeMap<&'static str, KindTally> = BTreeMap::new();
    for (topic, attempt) in attempts {
        for answer in &attempt.per_question {
            let Some(question) = topic.questions.iter().find(|it| it.id == answer.id) else {
                continue;
            };
            let kind = question.kind.label();
            let tally = tallies.entry(kind).or_insert_with(|| KindTally {
                kind: kind.to_owned(),
                ok: 0,
                partial: 0,
                miss: 0,
            });
            match answer.outcome {
                Outcome::Ok => tally.ok += 1,
                Outcome::Partial => tally.partial += 1,
                Outcome::Miss => tally.miss += 1,
            }
        }
    }
    tallies.into_values().collect()
}

fn actions(attempts: &[(&Topic, &Attempt)]) -> Vec<ActionTally> {
    let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    for (_, attempt) in attempts {
        let Some(action) = attempt.next_action else {
            continue;
        };
        *counts.entry(action.label()).or_insert(0) += 1;
    }
    let mut tallies: Vec<ActionTally> = counts
        .into_iter()
        .map(|(action, count)| ActionTally {
            action: action.to_owned(),
            count,
        })
        .collect();
    tallies.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.action.cmp(&right.action))
    });
    tallies
}

fn streak(topics: &[Topic], progress: &Progress) -> Streak {
    topics
        .iter()
        .filter_map(|topic| progress.state(&topic.id).map(|state| (topic, state)))
        .map(|(topic, state)| Streak {
            longest: longest(state),
            topic: topic.id.clone(),
        })
        .max_by(|left, right| {
            left.longest
                .cmp(&right.longest)
                .then_with(|| right.topic.cmp(&left.topic))
        })
        .filter(|streak| streak.longest > 0)
        .unwrap_or_default()
}

fn longest(state: &TopicState) -> u32 {
    let mut best = 0;
    let mut run = 0;
    for attempt in counted(&state.attempts) {
        run = if attempt.verdict == Verdict::Fail {
            run + 1
        } else {
            0
        };
        best = best.max(run);
    }
    best
}
