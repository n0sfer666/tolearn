use tolearn_core::Date;
use tolearn_core::plan::{Ahead, plan};
use tolearn_core::status::effective;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{AheadView, PlanIn, PlanOut, Span};

pub fn run(_context: &Context, input: &PlanIn) -> Result<PlanOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    let ahead = plan(&opened.scan.roadmap, &opened.scan.topics, &statuses, day);

    Ok(PlanOut {
        weekly_hours: ahead.weekly_hours,
        daily_hours: ahead.daily_hours,
        left: Span {
            min: ahead.left.min,
            max: ahead.left.max,
        },
        unknown: ahead.unknown,
        soonest: view(&ahead.soonest),
        latest: view(&ahead.latest),
    })
}

fn view(ahead: &Ahead) -> AheadView {
    AheadView {
        days: ahead.days,
        date: ahead.date.clone(),
    }
}
