use crate::exam::{Seen, Stage, stage};
use crate::ipc::types::{ExamLineView, ExamStateOut};
use crate::ipc::verdict::answer as graded;

pub fn view(seen: &Seen) -> ExamStateOut {
    let total = u32::try_from(seen.topic.questions.len()).unwrap_or_default();
    let Some(dialog) = &seen.dialog else {
        return ExamStateOut {
            open: false,
            stale: false,
            stage: Stage::Practice.label().to_owned(),
            asked: 0,
            hint_ready: false,
            log: Vec::new(),
            graded: Vec::new(),
            hinted: Vec::new(),
            seconds: 0,
            tokens: 0,
            verdict: None,
            total,
        };
    };
    let now = stage(dialog, &seen.topic);
    ExamStateOut {
        open: true,
        stale: dialog.stale(&seen.topic),
        stage: if dialog.verdict.is_some() {
            "done".to_owned()
        } else {
            now.label().to_owned()
        },
        asked: u32::try_from(dialog.graded.len()).unwrap_or_default(),
        hint_ready: matches!(now, Stage::Question(_)) && dialog.verdict.is_none(),
        log: dialog
            .log
            .iter()
            .map(|kept| ExamLineView {
                side: kept.side.clone(),
                text: kept.text.clone(),
            })
            .collect(),
        graded: dialog.answers().iter().map(graded).collect(),
        hinted: dialog.hinted.clone(),
        seconds: u32::try_from(dialog.seconds).unwrap_or(u32::MAX),
        tokens: dialog.tokens,
        verdict: dialog.verdict.clone(),
        total,
    }
}
