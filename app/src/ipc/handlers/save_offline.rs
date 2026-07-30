use tolearn_core::scan::Scan;
use tolearn_core::topic::Material;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{SaveOfflineIn, SaveOfflineOut};
use crate::ipc::{open, settings};
use crate::offline;

pub fn run(context: &Context, input: &SaveOfflineIn) -> Result<SaveOfflineOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let mut materials = wanted(&scan, input.topic.as_deref())?;
    if let Some(previous) = input.again.as_deref() {
        let broken = offline::failed(previous);
        materials.retain(|material| broken.contains(&material.url));
    }

    let settings = settings::stored(context)?;
    let total = u32::try_from(materials.len()).unwrap_or(u32::MAX);
    let job = offline::start(
        &context.offline(),
        settings.budget_bytes(),
        &scan.roadmap.id,
        materials,
    );
    Ok(SaveOfflineOut { job, total })
}

fn wanted(scan: &Scan, topic: Option<&str>) -> Result<Vec<Material>, IpcError> {
    let Some(topic) = topic else {
        return Ok(every(scan));
    };
    scan.topics
        .iter()
        .find(|document| document.id == topic)
        .map(|document| document.materials.clone())
        .ok_or_else(|| IpcError::unknown_topic(topic))
}

fn every(scan: &Scan) -> Vec<Material> {
    let mut seen = Vec::new();
    let mut materials = Vec::new();
    for material in scan.topics.iter().flat_map(|document| &document.materials) {
        if seen.contains(&material.url) {
            continue;
        }
        seen.push(material.url.clone());
        materials.push(material.clone());
    }
    materials
}
