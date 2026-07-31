use tolearn_core::scan::Scan;
use tolearn_core::topic::Material;
use tolearn_offline::fresh::Freshness;
use tolearn_offline::store::Held;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{SaveOfflineIn, SaveOfflineOut};
use crate::ipc::{open, settings};
use crate::offline::{self, Mode};

pub fn run(context: &Context, input: &SaveOfflineIn) -> Result<SaveOfflineOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let mut materials = wanted(&scan, input.topic.as_deref())?;
    let settings = settings::stored(context)?;
    let budget = settings.budget_bytes();

    let mode = match input.again.as_deref() {
        Some(previous) => {
            let broken = offline::failed(previous);
            materials.retain(|material| broken.contains(&material.url));
            Mode::Save
        }
        None => mode(context, budget, &materials),
    };

    let total = u32::try_from(materials.len()).unwrap_or(u32::MAX);
    let job = offline::start(
        &context.offline(),
        budget,
        &scan.roadmap.id,
        materials,
        mode,
    );
    Ok(SaveOfflineOut { job, total })
}

fn mode(context: &Context, budget: u64, materials: &[Material]) -> Mode {
    let seen = offline::seen(&context.offline(), budget);
    let held: Vec<Option<Held>> = materials
        .iter()
        .map(|material| seen.held(&material.url))
        .collect();
    match offline::state(materials, &held, offline::now()) {
        Freshness::Missing => Mode::Save,
        _ => Mode::Refresh,
    }
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
