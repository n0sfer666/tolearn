mod types;

pub use types::{Session, Step, Timer};

use crate::moment::Moment;

pub fn advance(session: &Session, step: Step, box_min: u32, now: &str) -> Session {
    let spent = spent(session, now);
    let expired = expired(session, box_min, spent);
    match step {
        Step::Reset => Session::default(),
        Step::Pause => Session {
            started_at: None,
            spent_sec: spent,
            expired,
        },
        Step::Start if session.started_at.is_some() => Session {
            expired,
            ..session.clone()
        },
        Step::Start => Session {
            started_at: Moment::parse(now).map(|_| now.to_owned()),
            spent_sec: session.spent_sec,
            expired,
        },
    }
}

pub fn timer(session: &Session, box_min: u32, now: &str) -> Timer {
    let spent = spent(session, now);
    Timer {
        spent_sec: spent,
        left_sec: i64::from(box_min) * 60 - i64::from(spent),
        running: session.started_at.is_some(),
        expired: expired(session, box_min, spent),
    }
}

fn expired(session: &Session, box_min: u32, spent: u32) -> bool {
    session.expired || i64::from(spent) >= i64::from(box_min) * 60
}

fn spent(session: &Session, now: &str) -> u32 {
    session.spent_sec.saturating_add(running_sec(session, now))
}

fn running_sec(session: &Session, now: &str) -> u32 {
    let Some(started) = session.started_at.as_deref().and_then(Moment::parse) else {
        return 0;
    };
    let Some(now) = Moment::parse(now) else {
        return 0;
    };
    u32::try_from(now.seconds_since(started)).unwrap_or(0)
}
