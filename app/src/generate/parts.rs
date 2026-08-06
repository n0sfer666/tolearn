use tolearn_core::bundle::validate;
use tolearn_core::generate::{PROGRESS_SCHEMA, ROADMAP_SCHEMA, TOPIC_SCHEMA, generated, pick};
use tolearn_core::progress::parse as read_progress;
use tolearn_core::roadmap::{Roadmap, TopicEntry, parse as read_roadmap};
use tolearn_core::topic::{Topic, parse as read_topic};

pub fn skeleton(answer: &str) -> Result<(Roadmap, String, String), Vec<String>> {
    let map_text = block(answer, ROADMAP_SCHEMA)?;
    let progress = block(answer, PROGRESS_SCHEMA)?;
    let map = read_roadmap(&map_text).map_err(spoken)?;
    read_progress(&progress).map_err(spoken)?;
    refused(validate(&map, &[]))?;
    Ok((map, map_text, progress))
}

pub fn one(
    map: &Roadmap,
    entry: &TopicEntry,
    answer: &str,
) -> Result<(Topic, String), Vec<String>> {
    let text = block(answer, TOPIC_SCHEMA)?;
    let topic = read_topic(&text).map_err(spoken)?;
    if topic.id != entry.id {
        return Err(vec![format!(
            "`id` темы — `{}`, а просили `{}`",
            topic.id, entry.id
        )]);
    }
    refused(validate(map, std::slice::from_ref(&topic)))?;
    Ok((topic, text))
}

pub fn whole(map_text: &str, topics: &[Topic]) -> Result<(Roadmap, String), Vec<String>> {
    let marked = generated(map_text);
    let map = read_roadmap(&marked).map_err(spoken)?;
    refused(validate(&map, topics))?;
    Ok((map, marked))
}

fn block(answer: &str, schema: &str) -> Result<String, Vec<String>> {
    pick(answer, schema).ok_or_else(|| {
        vec![format!(
            "в ответе нет блока, объявляющего `schema: {schema}`"
        )]
    })
}

fn refused(found: Vec<tolearn_core::bundle::Violation>) -> Result<(), Vec<String>> {
    if found.is_empty() {
        return Ok(());
    }
    Err(found.iter().map(ToString::to_string).collect())
}

fn spoken(error: impl ToString) -> Vec<String> {
    vec![error.to_string()]
}
