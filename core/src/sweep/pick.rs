use crate::date::Date;
use crate::progress::Progress;
use crate::roadmap::Roadmap;
use crate::status::{effective, is_done};
use crate::topic::Topic;

use super::types::Picked;

type Key = (u8, Option<Date>, String);

pub fn pool(roadmap: &Roadmap, topics: &[Topic], progress: &Progress, today: Date) -> Vec<Picked> {
    let statuses = effective(roadmap, topics, progress, today);
    let mut ready: Vec<(Key, Picked)> = topics
        .iter()
        .filter(|topic| !topic.questions.is_empty())
        .filter(|topic| statuses.get(&topic.id).is_some_and(is_done))
        .map(|topic| ranked(topic, progress, today))
        .collect();
    ready.sort_by(|left, right| left.0.cmp(&right.0));
    ready.into_iter().map(|(_, picked)| picked).collect()
}

fn ranked(topic: &Topic, progress: &Progress, today: Date) -> (Key, Picked) {
    let state = progress.state(&topic.id);
    let planned = state
        .and_then(|state| state.next_review_at.as_deref())
        .and_then(Date::parse);
    let waiting = planned.filter(|date| *date <= today);
    let key = match waiting {
        Some(date) => (0, Some(date), topic.id.clone()),
        None => (
            1,
            state
                .and_then(|state| state.passed_at.as_deref())
                .and_then(Date::parse),
            topic.id.clone(),
        ),
    };
    (
        key,
        Picked {
            topic: topic.id.clone(),
            title: topic.title.clone(),
            due: planned.map(|date| date.to_string()),
            overdue: waiting.is_some_and(|date| date < today),
        },
    )
}
