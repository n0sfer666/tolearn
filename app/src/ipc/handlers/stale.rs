use tolearn_core::Date;
use tolearn_core::stale::{Aging, Expired, digest};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{AgingView, ExpiredView, StaleIn, StaleOut};

pub fn run(_context: &Context, input: &StaleIn) -> Result<StaleOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let taken = digest(&opened.scan.roadmap, &opened.scan.topics, day);

    Ok(StaleOut {
        topics: taken.topics.iter().map(expired).collect(),
        materials: taken.materials.iter().map(aging).collect(),
    })
}

fn expired(topic: &Expired) -> ExpiredView {
    ExpiredView {
        topic: topic.topic.clone(),
        title: topic.title.clone(),
        verified_at: topic.verified_at.clone(),
        expired_at: topic.expired_at.clone(),
    }
}

fn aging(material: &Aging) -> AgingView {
    AgingView {
        topic: material.topic.clone(),
        topic_title: material.topic_title.clone(),
        title: material.title.clone(),
        url: material.url.clone(),
        stale: material.stale,
        delta: material.delta.clone(),
        covers_version: material.covers_version.clone(),
        pin: material.pin.label().to_owned(),
    }
}
