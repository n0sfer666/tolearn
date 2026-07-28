use tolearn_core::stats::stats;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{ActionView, KindView, StatsIn, StatsOut, StreakView};

pub fn run(_context: &Context, input: &StatsIn) -> Result<StatsOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let taken = stats(&opened.scan.topics, opened.document.progress());

    Ok(StatsOut {
        attempts: taken.attempts,
        enough: taken.enough,
        hinted: taken.hinted,
        hinted_share: taken.hinted_share,
        kinds: taken
            .kinds
            .iter()
            .map(|kind| KindView {
                kind: kind.kind.clone(),
                ok: kind.ok,
                partial: kind.partial,
                miss: kind.miss,
            })
            .collect(),
        actions: taken
            .actions
            .iter()
            .map(|action| ActionView {
                action: action.action.clone(),
                count: action.count,
            })
            .collect(),
        streak: StreakView {
            longest: taken.streak.longest,
            topic: taken.streak.topic.clone(),
        },
        calibration: taken.calibration.clone(),
    })
}
