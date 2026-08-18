mod types;

pub use types::Due;

use crate::date::Date;
use crate::progress::Progress;
use crate::topic::{Retention, Topic};

pub fn due(topics: &[Topic], progress: &Progress, today: Date) -> Vec<Due> {
    let mut queue: Vec<(Date, Due)> = topics
        .iter()
        .filter_map(|topic| planned(topic, progress, today))
        .collect();
    queue.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.topic.cmp(&right.1.topic)));
    queue.into_iter().map(|(_, item)| item).collect()
}

pub fn repeat(topic: &Topic, today: Date) -> Option<Date> {
    match topic.retention {
        Retention::None => None,
        _ => Some(today.plus_days(topic.revalidate_after_days)),
    }
}

fn planned(topic: &Topic, progress: &Progress, today: Date) -> Option<(Date, Due)> {
    let planned = progress.state(&topic.id)?.next_review_at.as_deref()?;
    let planned = Date::parse(planned)?;
    if planned > today {
        return None;
    }
    Some((
        planned,
        Due {
            topic: topic.id.clone(),
            title: topic.title.clone(),
            due: planned.to_string(),
            overdue: planned < today,
        },
    ))
}
