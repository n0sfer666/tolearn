mod flow;
mod run;
mod settle;
mod store;
mod words;

pub use flow::{Stage, stage};
pub use run::{Leg, Run};
pub use settle::Settled;

use tolearn_core::Date;
use tolearn_core::exam::{Artifact, Collected, verdict};
use tolearn_core::prompt::render;
use tolearn_core::sweep::{Picked, pool};
use tolearn_core::topic::Topic;

use crate::exam::Speaker;
use crate::ipc::open::{self, Opened};
use crate::ipc::verdict::read;
use crate::ipc::{Context, IpcError};

#[derive(Debug)]
pub struct Seen {
    pub ready: Vec<Picked>,
    pub run: Option<Run>,
    pub stale: bool,
}

pub fn look(context: &Context, bundle: &str, today: &str) -> Result<Seen, IpcError> {
    let opened = open::open(bundle)?;
    let ready = ready(&opened, today)?;
    let run = store::load(context, &opened.scan.roadmap.id);
    Ok(seen(&opened, ready, run))
}

pub fn start(
    context: &Context,
    bundle: &str,
    today: &str,
    topics: usize,
    restart: bool,
) -> Result<Seen, IpcError> {
    let opened = open::open(bundle)?;
    let ready = ready(&opened, today)?;
    let kept = store::load(context, &opened.scan.roadmap.id);
    let run = match kept {
        Some(kept) if !restart && !kept.done && !kept.stale(&opened.scan.topics) => kept,
        _ => opened_run(&opened, &ready, topics)?,
    };
    store::save(context, &run)?;
    Ok(seen(&opened, ready, Some(run)))
}

pub fn answer(context: &Context, bundle: &str, today: &str, text: &str) -> Result<Seen, IpcError> {
    stepped(context, bundle, today, |speaker, run, topics| {
        flow::said(speaker, run, topics, text)
    })
}

pub fn hint(context: &Context, bundle: &str, today: &str) -> Result<Seen, IpcError> {
    stepped(context, bundle, today, flow::hinted)
}

pub fn finish(context: &Context, bundle: &str, today: &str) -> Result<Seen, IpcError> {
    let opened = open::open(bundle)?;
    let ready = ready(&opened, today)?;
    let mut run = loaded(context, &opened.scan.roadmap.id)?;
    let speaker = Speaker::new(context)?;
    let judged = judged(&speaker, &mut run, &opened, today);
    run.done = judged.is_ok();
    store::save(context, &run)?;
    judged?;
    Ok(seen(&opened, ready, Some(run)))
}

pub fn accept(context: &Context, bundle: &str, today: &str) -> Result<Vec<Settled>, IpcError> {
    let mut opened = open::open(bundle)?;
    let run = loaded(context, &opened.scan.roadmap.id)?;
    settle::accept(context, &mut opened, &run, day(today)?)
}

fn stepped(
    context: &Context,
    bundle: &str,
    today: &str,
    walk: impl FnOnce(&Speaker, &mut Run, &[Topic]) -> Result<(), IpcError>,
) -> Result<Seen, IpcError> {
    let opened = open::open(bundle)?;
    let ready = ready(&opened, today)?;
    let mut run = loaded(context, &opened.scan.roadmap.id)?;
    if run.done {
        return Err(IpcError::new(
            "sweep.finished",
            "прогон уже завершён — вердикты вынесены".to_owned(),
        ));
    }
    let speaker = Speaker::new(context)?;
    walk(&speaker, &mut run, &opened.scan.topics)?;
    store::save(context, &run)?;
    Ok(seen(&opened, ready, Some(run)))
}

fn judged(speaker: &Speaker, run: &mut Run, opened: &Opened, today: &str) -> Result<(), IpcError> {
    for index in 0..run.legs.len() {
        if run.legs[index].verdict.is_some() || run.legs[index].graded.is_empty() {
            continue;
        }
        let topic = read(opened, &run.legs[index].topic)?;
        let asked = verdict(
            &examiner(opened, topic)?,
            topic,
            &Collected {
                graded: run.legs[index].answers(),
                hinted: run.legs[index].hinted.clone(),
                artifact: Artifact::Aside,
                failed_checks: Vec::new(),
                today: today.to_owned(),
            },
        );
        let heard = speaker.ask(&asked)?;
        run.spent(heard.seconds, heard.tokens);
        run.legs[index].verdict = Some(heard.text);
    }
    Ok(())
}

fn opened_run(opened: &Opened, ready: &[Picked], topics: usize) -> Result<Run, IpcError> {
    let picked: Vec<Picked> = ready.iter().take(topics.max(1)).cloned().collect();
    if picked.is_empty() {
        return Err(IpcError::new(
            "sweep.nothing",
            "повторять нечего: в программе нет пройденных тем с вопросами".to_owned(),
        ));
    }
    Ok(flow::opened(
        &opened.scan.roadmap.id,
        &picked,
        &opened.scan.topics,
    ))
}

fn loaded(context: &Context, roadmap: &str) -> Result<Run, IpcError> {
    store::load(context, roadmap).ok_or_else(|| {
        IpcError::new(
            "sweep.not-started",
            "прогон по этой программе не начат".to_owned(),
        )
    })
}

fn ready(opened: &Opened, today: &str) -> Result<Vec<Picked>, IpcError> {
    Ok(pool(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day(today)?,
    ))
}

fn seen(opened: &Opened, ready: Vec<Picked>, run: Option<Run>) -> Seen {
    Seen {
        stale: run
            .as_ref()
            .is_some_and(|run| run.stale(&opened.scan.topics)),
        ready,
        run,
    }
}

fn examiner(opened: &Opened, topic: &Topic) -> Result<String, IpcError> {
    let template = open::text(&opened.scan.root.join("examiner.md"))?;
    Ok(render(&template, topic, &opened.scan.roadmap)?)
}

fn day(today: &str) -> Result<Date, IpcError> {
    Date::parse(today).ok_or_else(|| IpcError::malformed_date(today))
}
