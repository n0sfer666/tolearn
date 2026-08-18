mod types;

pub use types::{Aging, Digest, Expired, Pin};

use crate::Date;
use crate::roadmap::Roadmap;
use crate::topic::{Material, Topic};

pub fn digest(roadmap: &Roadmap, topics: &[Topic], today: Date) -> Digest {
    let mut expired: Vec<Expired> = topics.iter().flat_map(|topic| aged(topic, today)).collect();
    expired.sort_by(|left, right| {
        (&left.expired_at, &left.topic).cmp(&(&right.expired_at, &right.topic))
    });

    Digest {
        topics: expired,
        materials: topics
            .iter()
            .flat_map(|topic| {
                topic
                    .materials
                    .iter()
                    .filter_map(|material| aging(roadmap, topic, material))
            })
            .collect(),
    }
}

fn aged(topic: &Topic, today: Date) -> Option<Expired> {
    let verified = Date::parse(&topic.verified_at)?;
    let expired = verified.plus_days(topic.revalidate_after_days);
    if expired > today {
        return None;
    }
    Some(Expired {
        topic: topic.id.clone(),
        title: topic.title.clone(),
        verified_at: topic.verified_at.clone(),
        expired_at: expired.to_string(),
    })
}

fn aging(roadmap: &Roadmap, topic: &Topic, material: &Material) -> Option<Aging> {
    let pin = pinned(roadmap, material.covers_version.as_deref());
    if !material.stale && material.delta.is_none() && pin != Pin::Unknown {
        return None;
    }
    Some(Aging {
        topic: topic.id.clone(),
        topic_title: topic.title.clone(),
        title: material.title.clone(),
        url: material.url.clone(),
        stale: material.stale,
        delta: material.delta.clone(),
        covers_version: material.covers_version.clone(),
        pin,
    })
}

fn pinned(roadmap: &Roadmap, claimed: Option<&str>) -> Pin {
    let Some(claimed) = claimed else {
        return Pin::Absent;
    };
    if roadmap.version_pins.values().any(|pin| pin == claimed) {
        return Pin::Known;
    }
    Pin::Unknown
}
