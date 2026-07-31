use tolearn_core::Date;
use tolearn_core::roadmap::Roadmap;
use tolearn_core::status::{Statuses, effective, is_done};
use tolearn_core::topic::{Check, Material, Practice, Question, Topic};

use tolearn_offline::store::Held;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{
    CheckView, ExamView, Link, MaterialView, PracticeView, QuestionView, Span, TopicIn, TopicOut,
    UnloadView,
};
use crate::ipc::{open, settings};
use crate::offline::{self, label};

pub fn run(context: &Context, input: &TopicIn) -> Result<TopicOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let document = opened
        .scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;
    let settings = settings::stored(context)?;
    let seen = offline::seen(&context.offline(), settings.budget_bytes());
    let held: Vec<Option<Held>> = document
        .materials
        .iter()
        .map(|source| seen.held(&source.url))
        .collect();
    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );

    Ok(TopicOut {
        id: document.id.clone(),
        title: document.title.clone(),
        program: opened.scan.roadmap.title.clone(),
        stage: document.stage,
        checkpoint: is_checkpoint(&opened.scan.roadmap, &document.id),
        status: statuses
            .get(&document.id)
            .map(|status| status.label().to_owned())
            .unwrap_or_default(),
        hours: Span {
            min: document.est_hours.min,
            max: document.est_hours.max,
        },
        blocked_by: blocked_by(&opened.scan.roadmap, document, &statuses),
        verified_at: document.verified_at.clone(),
        outdated: outdated(document, day),
        outcomes: document.outcomes.clone(),
        misconceptions: document.misconceptions.clone(),
        materials: document
            .materials
            .iter()
            .zip(&held)
            .map(|(source, held)| material(source, held.as_ref()))
            .collect(),
        unload: unload(&document.materials, &held),
        practice: practice(&document.practice),
        questions: document.questions.iter().map(question).collect(),
        exam: ExamView {
            focus: document.exam.focus.clone(),
            artifact_required: document.exam.artifact_required,
            max_exchanges: document.exam.max_exchanges,
        },
    })
}

fn is_checkpoint(roadmap: &Roadmap, id: &str) -> bool {
    roadmap.stages.iter().any(|stage| stage.checkpoint == id)
}

fn outdated(document: &Topic, today: Date) -> bool {
    Date::parse(&document.verified_at)
        .map(|verified| verified.plus_days(document.revalidate_after_days) <= today)
        .unwrap_or(false)
}

fn blocked_by(roadmap: &Roadmap, document: &Topic, statuses: &Statuses) -> Vec<Link> {
    document
        .depends_on
        .iter()
        .filter(|needed| !statuses.get(needed).is_some_and(is_done))
        .map(|needed| Link {
            id: needed.clone(),
            title: title_of(roadmap, needed),
        })
        .collect()
}

fn title_of(roadmap: &Roadmap, id: &str) -> String {
    roadmap
        .topics
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.title.clone())
        .unwrap_or_else(|| id.to_owned())
}

fn unload(materials: &[Material], held: &[Option<Held>]) -> UnloadView {
    let state = offline::state(materials, held, offline::now());
    UnloadView {
        state: state.name().to_owned(),
        checked_at: state.at(),
    }
}

fn material(source: &Material, held: Option<&Held>) -> MaterialView {
    MaterialView {
        title: source.title.clone(),
        url: source.url.clone(),
        kind: source.kind.label().to_owned(),
        tier: source.tier.label().to_owned(),
        lang: source.lang.clone(),
        stale: source.stale,
        delta: source.delta.clone(),
        offline: label(held).to_owned(),
        note: source.note.clone(),
    }
}

fn practice(source: &Practice) -> PracticeView {
    PracticeView {
        kind: source.kind.label().to_owned(),
        tier: source.tier.label().to_owned(),
        task: source.task.clone(),
        deliverable: source.deliverable.clone(),
        starting_point: source.starting_point.clone(),
        fallback: source.fallback.clone(),
        time_box_min: source.time_box_min,
        smoke_checked: source.smoke_checked,
        constraints: source.constraints.iter().map(check).collect(),
        acceptance: source.acceptance.iter().map(check).collect(),
    }
}

fn check(source: &Check) -> CheckView {
    CheckView {
        id: source.id.clone(),
        claim: source.claim.clone(),
        check: source.check.clone(),
        expect: source.expect.clone(),
    }
}

fn question(source: &Question) -> QuestionView {
    QuestionView {
        id: source.id.clone(),
        kind: source.kind.label().to_owned(),
        text: source.text.clone(),
    }
}
