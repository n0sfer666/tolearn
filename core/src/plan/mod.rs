mod types;

pub use types::{Ahead, Plan};

use crate::Hours;
use crate::date::Date;
use crate::roadmap::{Priority, Roadmap};
use crate::status::{Statuses, is_done};
use crate::topic::Topic;

pub fn plan(roadmap: &Roadmap, topics: &[Topic], statuses: &Statuses, today: Date) -> Plan {
    let mut left = Hours::default();
    let mut unknown = 0;

    for entry in &roadmap.topics {
        if entry.priority == Priority::Optional {
            continue;
        }
        if !topics.iter().any(|topic| topic.id == entry.id) {
            unknown += 1;
            continue;
        }
        if statuses.get(&entry.id).is_some_and(is_done) {
            continue;
        }
        left.min += entry.est_hours.min;
        left.max += entry.est_hours.max;
    }

    let daily = f64::from(roadmap.weekly_hours) / 7.0;
    Plan {
        weekly_hours: roadmap.weekly_hours,
        daily_hours: daily,
        left,
        unknown,
        soonest: ahead(left.min, daily, today),
        latest: ahead(left.max, daily, today),
    }
}

fn ahead(hours: u32, daily: f64, today: Date) -> Ahead {
    let days = days(hours, daily);
    Ahead {
        days,
        date: today.plus_days(days).to_string(),
    }
}

fn days(hours: u32, daily: f64) -> u32 {
    if daily <= 0.0 {
        return 0;
    }
    (f64::from(hours) / daily).ceil() as u32
}
