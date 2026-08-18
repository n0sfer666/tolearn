use crate::generate::{Staged, Summary, look};
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{GenerateStateIn, GenerateStateOut, StagedView, SummaryView};

pub fn run(_context: &Context, input: &GenerateStateIn) -> Result<GenerateStateOut, IpcError> {
    let live = look(&input.job).ok_or_else(|| unknown(&input.job))?;
    Ok(GenerateStateOut {
        step: live.step,
        total: count(live.total),
        done: count(live.done),
        attempt: live.attempt,
        rounds: crate::generate::ROUNDS,
        retry: live.retry,
        tries: crate::generate::TRIES,
        current: live.current,
        waiting: live.waiting,
        finished: live.finished,
        cancelled: live.cancelled,
        refused: live.refused,
        missed: live.missed,
        seconds: live.seconds,
        step_seconds: live.step_seconds,
        chars: live.chars,
        ticks: live.ticks,
        tail: live.tail,
        tokens: live.tokens,
        summary: live.summary.map(seen),
    })
}

fn seen(summary: Summary) -> SummaryView {
    SummaryView {
        id: summary.id,
        title: summary.title,
        topics: count(summary.topics),
        hours_min: summary.hours_min,
        hours_max: summary.hours_max,
        stages: summary.stages.into_iter().map(staged).collect(),
    }
}

fn staged(stage: Staged) -> StagedView {
    StagedView {
        n: stage.n,
        title: stage.title,
        topics: count(stage.topics),
        first: stage.first,
    }
}

fn count(many: usize) -> u32 {
    u32::try_from(many).unwrap_or(u32::MAX)
}

pub fn unknown(job: &str) -> IpcError {
    IpcError::new(
        "generate.unknown-job",
        format!("генерации `{job}` нет среди начатых"),
    )
}
