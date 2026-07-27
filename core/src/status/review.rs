use crate::date::Date;
use crate::topic::{Retention, Topic};

pub fn next_review_at(topic: &Topic, passed_at: Option<Date>) -> Option<Date> {
    let base = match topic.retention {
        Retention::None => return None,
        Retention::ByUse => Date::parse(&topic.verified_at)?,
        Retention::BySchedule => passed_at?,
    };
    Some(base.plus_days(topic.revalidate_after_days))
}
