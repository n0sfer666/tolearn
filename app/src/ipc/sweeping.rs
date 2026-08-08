use tolearn_core::sweep::Picked;

use crate::ipc::types::{
    ExamLineView, SweepLegView, SweepPickView, SweepSettledView, SweepStateOut,
};
use crate::sweep::{Leg, Seen, Settled, Stage, stage};

pub fn view(seen: &Seen) -> SweepStateOut {
    let ready = seen.ready.iter().map(picked).collect();
    let Some(run) = &seen.run else {
        return SweepStateOut {
            open: false,
            stale: false,
            done: false,
            stage: Stage::Over.label().to_owned(),
            asked: 0,
            total: 0,
            hint_ready: false,
            ready,
            legs: Vec::new(),
            log: Vec::new(),
            seconds: 0,
            tokens: 0,
        };
    };
    let now = stage(run);
    SweepStateOut {
        open: true,
        stale: seen.stale,
        done: run.done,
        stage: if run.done {
            "done".to_owned()
        } else {
            now.label().to_owned()
        },
        asked: count(run.asked()),
        total: count(run.total()),
        hint_ready: matches!(now, Stage::Question(_)) && !run.done,
        ready,
        legs: run.legs.iter().map(leg).collect(),
        log: run
            .log
            .iter()
            .map(|kept| ExamLineView {
                side: kept.side.clone(),
                text: kept.text.clone(),
            })
            .collect(),
        seconds: u32::try_from(run.seconds).unwrap_or(u32::MAX),
        tokens: run.tokens,
    }
}

pub fn settled(settled: &[Settled]) -> Vec<SweepSettledView> {
    settled
        .iter()
        .map(|item| SweepSettledView {
            topic: item.topic.clone(),
            title: item.title.clone(),
            result: item.result.clone(),
            status: item.status.clone(),
        })
        .collect()
}

fn picked(pick: &Picked) -> SweepPickView {
    SweepPickView {
        topic: pick.topic.clone(),
        title: pick.title.clone(),
        due: pick.due.clone(),
        overdue: pick.overdue,
    }
}

fn leg(leg: &Leg) -> SweepLegView {
    SweepLegView {
        topic: leg.topic.clone(),
        title: leg.title.clone(),
        asked: count(leg.graded.len()),
        total: count(leg.total),
        verdict: leg.verdict.clone(),
    }
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
