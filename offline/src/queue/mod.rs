mod report;
mod strategy;

pub use report::{Report, Skip};
pub use strategy::{Step, Strategy, plan};

use std::sync::atomic::{AtomicBool, Ordering};

use tolearn_core::topic::{Liveness, Material};

pub trait Saver {
    fn save(&self, material: &Material, how: Strategy) -> Result<u64, String>;
}

pub fn unload(materials: &[Material], saver: &dyn Saver, stop: &AtomicBool) -> Report {
    let mut report = Report::default();
    for material in materials {
        let how = match plan(material) {
            Step::Skip(why) => {
                report.skipped.push((material.url.clone(), why));
                continue;
            }
            Step::Take(how) => how,
        };
        if stop.load(Ordering::Relaxed) {
            report.cancelled = true;
            return report;
        }
        match saver.save(material, how) {
            Ok(bytes) => {
                report.bytes += bytes;
                report.saved.push(material.url.clone());
            }
            Err(reason) => report.failed.push((material.url.clone(), reason)),
        }
    }
    report
}

pub fn again(
    materials: &[Material],
    after: &Report,
    saver: &dyn Saver,
    stop: &AtomicBool,
) -> Report {
    let broken = after.failed_urls();
    let again: Vec<Material> = materials
        .iter()
        .filter(|material| broken.contains(&material.url.as_str()))
        .cloned()
        .collect();
    unload(&again, saver, stop)
}

fn closed(liveness: Liveness) -> Option<Skip> {
    match liveness {
        Liveness::Ok => None,
        Liveness::Paywall => Some(Skip::Paywall),
        Liveness::LoginRequired => Some(Skip::LoginRequired),
    }
}
