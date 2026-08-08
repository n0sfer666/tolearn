use tolearn_core::exam::{Next, Side, hint, step, turn};
use tolearn_core::sweep::Picked;
use tolearn_core::topic::{Question, Topic};

use crate::exam::{Speaker, refused};
use crate::ipc::IpcError;

use super::run::Run;
use super::words;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Question(usize),
    Over,
}

impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Question(_) => "question",
            Self::Over => "over",
        }
    }
}

pub fn stage(run: &Run) -> Stage {
    let count = run.legs.len();
    if count == 0 {
        return Stage::Over;
    }
    (0..count)
        .map(|step| (run.at + step) % count)
        .find(|index| run.legs[*index].left())
        .map_or(Stage::Over, Stage::Question)
}

pub fn opened(roadmap: &str, picked: &[Picked], topics: &[Topic]) -> Run {
    let mut run = Run::fresh(roadmap, picked, topics);
    let opening = words::opening(run.legs.len(), run.total());
    run.tell(Side::Examiner, &opening);
    ask(&mut run, topics);
    run
}

pub fn said(
    speaker: &Speaker,
    run: &mut Run,
    topics: &[Topic],
    text: &str,
) -> Result<(), IpcError> {
    let Stage::Question(at) = stage(run) else {
        return Err(IpcError::new(
            "sweep.over",
            "вопросы кончились — прогон пора завершать".to_owned(),
        ));
    };
    run.tell(Side::Student, text);
    let (topic, question) = asked(run, topics, at)?;
    let helped = run.legs[at].hinted.contains(&question.id);
    let heard = speaker.ask(&turn(topic, question, &run.said(), helped))?;
    run.spent(heard.seconds, heard.tokens);
    let told = step(&heard.text, question).map_err(refused)?;
    if told.next == Next::FollowUp && !run.followed {
        run.followed = true;
        run.tell(Side::Examiner, &told.say);
        return Ok(());
    }
    run.grade(at, &told.answer);
    close(run, topics, &told.say);
    Ok(())
}

pub fn hinted(speaker: &Speaker, run: &mut Run, topics: &[Topic]) -> Result<(), IpcError> {
    let Stage::Question(at) = stage(run) else {
        return Err(IpcError::new(
            "sweep.no-hint",
            "подсказка бывает только по вопросу темы".to_owned(),
        ));
    };
    let (topic, question) = asked(run, topics, at)?;
    let heard = speaker.ask(&hint(topic, question, &run.said()))?;
    let id = question.id.clone();
    run.spent(heard.seconds, heard.tokens);
    run.tell(Side::Examiner, &heard.text);
    if let Some(leg) = run.legs.get_mut(at)
        && !leg.hinted.contains(&id)
    {
        leg.hinted.push(id);
    }
    Ok(())
}

fn close(run: &mut Run, topics: &[Topic], say: &str) {
    if !say.trim().is_empty() {
        run.tell(Side::Examiner, say);
    }
    run.turned();
    ask(run, topics);
}

fn ask(run: &mut Run, topics: &[Topic]) {
    let Stage::Question(at) = stage(run) else {
        run.tell(Side::Examiner, words::OVER);
        return;
    };
    let Ok((_, question)) = asked(run, topics, at) else {
        run.tell(Side::Examiner, words::OVER);
        return;
    };
    let text = words::question(&run.legs[at].title, &question.text);
    run.tell(Side::Examiner, &text);
}

fn asked<'a>(
    run: &Run,
    topics: &'a [Topic],
    at: usize,
) -> Result<(&'a Topic, &'a Question), IpcError> {
    let leg = run
        .legs
        .get(at)
        .ok_or_else(|| IpcError::new("sweep.no-leg", "тема прогона потерялась".to_owned()))?;
    let topic = topics
        .iter()
        .find(|topic| topic.id == leg.topic)
        .ok_or_else(|| IpcError::unknown_topic(&leg.topic))?;
    let question = topic
        .questions
        .get(leg.at)
        .ok_or_else(|| IpcError::new("sweep.no-question", "вопрос потерялся".to_owned()))?;
    Ok((topic, question))
}
