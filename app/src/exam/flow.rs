use tolearn_core::exam::{Artifact, Next, Side, practice, shown, step, turn};
use tolearn_core::topic::Topic;

use crate::ipc::IpcError;

use super::dialog::Dialog;
use super::talk::Speaker;
use super::words;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Practice,
    Question(usize),
    Over,
}

impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Practice => "practice",
            Self::Question(_) => "question",
            Self::Over => "over",
        }
    }
}

pub fn stage(dialog: &Dialog, topic: &Topic) -> Stage {
    if topic.exam.artifact_required && dialog.artifact.is_none() {
        return Stage::Practice;
    }
    if dialog.at < topic.questions.len() {
        return Stage::Question(dialog.at);
    }
    Stage::Over
}

pub fn opened(topic: &Topic) -> Dialog {
    let mut dialog = Dialog::fresh(topic);
    dialog.tell(Side::Examiner, &words::opening(topic));
    ask(&mut dialog, topic);
    dialog
}

pub fn said(
    speaker: &Speaker,
    dialog: &mut Dialog,
    topic: &Topic,
    text: &str,
) -> Result<(), IpcError> {
    let now = stage(dialog, topic);
    if now == Stage::Over {
        return Err(IpcError::new(
            "exam.over",
            "вопросы кончились — зачёт пора завершать".to_owned(),
        ));
    }
    dialog.tell(Side::Student, text);
    dialog.exchanges += 1;
    match now {
        Stage::Practice => accepted(speaker, dialog, topic),
        Stage::Question(at) => graded(speaker, dialog, topic, at),
        Stage::Over => Ok(()),
    }
}

pub fn hinted(speaker: &Speaker, dialog: &mut Dialog, topic: &Topic) -> Result<(), IpcError> {
    let Stage::Question(at) = stage(dialog, topic) else {
        return Err(IpcError::new(
            "exam.no-hint",
            "подсказка бывает только по вопросу темы".to_owned(),
        ));
    };
    let question = &topic.questions[at];
    let heard = speaker.ask(&tolearn_core::exam::hint(topic, question, &dialog.said()))?;
    dialog.spent(heard.seconds, heard.tokens);
    dialog.tell(Side::Examiner, &heard.text);
    if !dialog.hinted.contains(&question.id) {
        dialog.hinted.push(question.id.clone());
    }
    Ok(())
}

fn accepted(speaker: &Speaker, dialog: &mut Dialog, topic: &Topic) -> Result<(), IpcError> {
    let heard = speaker.ask(&practice(topic, &dialog.said()))?;
    dialog.spent(heard.seconds, heard.tokens);
    let told = shown(&heard.text).map_err(refused)?;
    if told.next == Next::FollowUp && follows(dialog, topic) {
        dialog.followed = true;
        dialog.tell(Side::Examiner, &told.say);
        return Ok(());
    }
    dialog.artifact = Some(told.artifact.label().to_owned());
    dialog.failed_checks = told.failed_checks;
    close(dialog, topic, &told.say);
    Ok(())
}

fn graded(
    speaker: &Speaker,
    dialog: &mut Dialog,
    topic: &Topic,
    at: usize,
) -> Result<(), IpcError> {
    let question = &topic.questions[at];
    let hinted = dialog.hinted.contains(&question.id);
    let heard = speaker.ask(&turn(topic, question, &dialog.said(), hinted))?;
    dialog.spent(heard.seconds, heard.tokens);
    let told = step(&heard.text, question).map_err(refused)?;
    if told.next == Next::FollowUp && follows(dialog, topic) {
        dialog.followed = true;
        dialog.tell(Side::Examiner, &told.say);
        return Ok(());
    }
    dialog.grade(&told.answer);
    dialog.at = at + 1;
    close(dialog, topic, &told.say);
    Ok(())
}

fn close(dialog: &mut Dialog, topic: &Topic, say: &str) {
    if !say.trim().is_empty() {
        dialog.tell(Side::Examiner, say);
    }
    dialog.turned();
    ask(dialog, topic);
}

fn ask(dialog: &mut Dialog, topic: &Topic) {
    match stage(dialog, topic) {
        Stage::Practice => dialog.tell(Side::Examiner, &words::practice(topic)),
        Stage::Question(at) => {
            let text = topic.questions[at].text.clone();
            dialog.tell(Side::Examiner, &text);
        }
        Stage::Over => dialog.tell(Side::Examiner, words::OVER),
    }
}

fn follows(dialog: &Dialog, topic: &Topic) -> bool {
    !dialog.followed && dialog.exchanges < topic.exam.max_exchanges
}

pub fn artifact(dialog: &Dialog, topic: &Topic) -> Artifact {
    if !topic.exam.artifact_required {
        return Artifact::Skipped;
    }
    dialog
        .artifact
        .as_deref()
        .and_then(Artifact::read)
        .unwrap_or(Artifact::None)
}

pub fn refused(error: tolearn_core::exam::StepError) -> IpcError {
    IpcError::new(error.code(), error.to_string())
}
