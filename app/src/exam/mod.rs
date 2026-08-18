mod dialog;
mod flow;
mod store;
mod talk;
mod words;

pub use dialog::{Dialog, Grade, Kept, answers, fingerprint, scored};
pub use flow::{Stage, refused, stage};
pub use store::{forget, safe};
pub use talk::Speaker;

use tolearn_core::exam::{Collected, verdict};
use tolearn_core::prompt::render;
use tolearn_core::scan::Scan;
use tolearn_core::topic::Topic;

use crate::ipc::open;
use crate::ipc::{Context, IpcError};

#[derive(Debug)]
pub struct Seen {
    pub topic: Topic,
    pub dialog: Option<Dialog>,
}

pub fn look(context: &Context, bundle: &str, id: &str) -> Result<Seen, IpcError> {
    let (scan, topic) = found(bundle, id)?;
    let dialog = store::load(context, &scan.roadmap.id, &topic.id);
    Ok(Seen { topic, dialog })
}

pub fn start(context: &Context, bundle: &str, id: &str, restart: bool) -> Result<Seen, IpcError> {
    let (scan, topic) = found(bundle, id)?;
    let kept = store::load(context, &scan.roadmap.id, &topic.id);
    let dialog = match kept {
        Some(kept) if !restart && !kept.stale(&topic) && kept.verdict.is_none() => kept,
        _ => flow::opened(&topic),
    };
    store::save(context, &scan.roadmap.id, &dialog)?;
    Ok(Seen {
        topic,
        dialog: Some(dialog),
    })
}

pub fn answer(context: &Context, bundle: &str, id: &str, text: &str) -> Result<Seen, IpcError> {
    stepped(context, bundle, id, |speaker, dialog, topic, _| {
        flow::said(speaker, dialog, topic, text)
    })
}

pub fn hint(context: &Context, bundle: &str, id: &str) -> Result<Seen, IpcError> {
    stepped(context, bundle, id, |speaker, dialog, topic, _| {
        flow::hinted(speaker, dialog, topic)
    })
}

pub fn finish(context: &Context, bundle: &str, id: &str, today: &str) -> Result<Seen, IpcError> {
    stepped(context, bundle, id, |speaker, dialog, topic, scan| {
        let examiner = examiner(scan, topic)?;
        let got = Collected {
            graded: dialog.answers(),
            hinted: dialog.hinted.clone(),
            artifact: flow::artifact(dialog, topic),
            failed_checks: dialog.failed_checks.clone(),
            today: today.to_owned(),
        };
        let heard = speaker.ask(&verdict(&examiner, topic, &got))?;
        dialog.spent(heard.seconds, heard.tokens);
        dialog.verdict = Some(heard.text);
        Ok(())
    })
}

fn stepped(
    context: &Context,
    bundle: &str,
    id: &str,
    walk: impl FnOnce(&Speaker, &mut Dialog, &Topic, &Scan) -> Result<(), IpcError>,
) -> Result<Seen, IpcError> {
    let (scan, topic) = found(bundle, id)?;
    let mut dialog = store::load(context, &scan.roadmap.id, &topic.id).ok_or_else(|| {
        IpcError::new("exam.not-started", "зачёт по этой теме не начат".to_owned())
    })?;
    if dialog.verdict.is_some() {
        return Err(IpcError::new(
            "exam.finished",
            "зачёт уже завершён — вердикт вынесен".to_owned(),
        ));
    }
    let speaker = Speaker::new(context)?;
    walk(&speaker, &mut dialog, &topic, &scan)?;
    store::save(context, &scan.roadmap.id, &dialog)?;
    Ok(Seen {
        topic,
        dialog: Some(dialog),
    })
}

fn examiner(scan: &Scan, topic: &Topic) -> Result<String, IpcError> {
    let template = open::text(&scan.root.join("examiner.md"))?;
    Ok(render(&template, topic, &scan.roadmap)?)
}

fn found(bundle: &str, id: &str) -> Result<(Scan, Topic), IpcError> {
    let scan = open::read(bundle)?;
    let topic = scan
        .topics
        .iter()
        .find(|topic| topic.id == id)
        .cloned()
        .ok_or_else(|| IpcError::unknown_topic(id))?;
    Ok((scan, topic))
}
