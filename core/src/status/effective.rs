use crate::date::Date;
use crate::progress::{Progress, Status};
use crate::roadmap::Roadmap;
use crate::topic::Topic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statuses(Vec<(String, Status)>);

impl Statuses {
    pub fn get(&self, topic: &str) -> Option<Status> {
        self.0
            .iter()
            .find(|(id, _)| id == topic)
            .map(|(_, status)| *status)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, Status)> {
        self.0.iter()
    }
}

pub fn effective(
    roadmap: &Roadmap,
    topics: &[Topic],
    progress: &Progress,
    today: Date,
) -> Statuses {
    let settled: Vec<(String, Status)> = roadmap
        .topics
        .iter()
        .map(|entry| {
            let document = topics.iter().find(|topic| topic.id == entry.id);
            (
                entry.id.clone(),
                settle(roadmap, &entry.id, document, progress, today),
            )
        })
        .collect();

    let statuses = settled
        .iter()
        .map(|(id, status)| {
            let document = topics.iter().find(|topic| &topic.id == id);
            (id.clone(), block(*status, document, &settled))
        })
        .collect();

    Statuses(statuses)
}

fn settle(
    roadmap: &Roadmap,
    id: &str,
    document: Option<&Topic>,
    progress: &Progress,
    today: Date,
) -> Status {
    let recorded = match progress.state(id).map(|state| state.status) {
        Some(Status::Blocked) | None => Status::Todo,
        Some(status) => status,
    };
    let status = if roadmap.calibration.passed_out.iter().any(|it| it == id) {
        Status::PassedOut
    } else {
        recorded
    };
    match document {
        Some(topic) if is_done(status) && went_stale(topic, today) => Status::StalePassed,
        _ => status,
    }
}

fn block(status: Status, document: Option<&Topic>, settled: &[(String, Status)]) -> Status {
    let Some(topic) = document else {
        return status;
    };
    if is_done(status) {
        return status;
    }
    let waiting = topic.depends_on.iter().any(|needed| {
        !settled
            .iter()
            .any(|(id, status)| id == needed && is_done(*status))
    });
    if waiting { Status::Blocked } else { status }
}

fn went_stale(topic: &Topic, today: Date) -> bool {
    Date::parse(&topic.verified_at)
        .map(|verified| verified.plus_days(topic.revalidate_after_days) <= today)
        .unwrap_or(false)
}

pub fn is_done(status: Status) -> bool {
    matches!(
        status,
        Status::Passed | Status::PassedOut | Status::StalePassed
    )
}
