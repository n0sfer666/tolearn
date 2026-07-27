use super::effective::is_done;
use super::review::next_review_at;
use crate::date::Date;
use crate::progress::{Mark, Status};
use crate::topic::Topic;

pub fn manual(status: Status, at: &str, today: Date, topic: &Topic) -> Mark {
    let passed_at = is_done(status).then_some(today);
    Mark {
        status,
        at: at.to_owned(),
        passed_at: passed_at.map(|date| date.to_string()),
        next_review_at: passed_at
            .and_then(|date| next_review_at(topic, Some(date)))
            .map(|date| date.to_string()),
        note: None,
    }
}
