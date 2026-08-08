use tolearn_core::bundle::{Violation, ordered, tracked, validate};
use tolearn_core::generate::{
    PROGRESS_SCHEMA, ROADMAP_SCHEMA, TOPIC_SCHEMA, generated, pick, stamped,
};
use tolearn_core::progress::parse as read_progress;
use tolearn_core::roadmap::{Roadmap, TopicEntry, parse as read_roadmap};
use tolearn_core::topic::{Topic, parse as read_topic};

pub struct Refused {
    pub why: Vec<String>,
    pub topics: Vec<String>,
}

pub fn skeleton(answer: &str, today: &str) -> Result<(Roadmap, String, String), Vec<String>> {
    let map_text = stamped(&block(answer, ROADMAP_SCHEMA)?, today);
    let text = block(answer, PROGRESS_SCHEMA)?;
    let map = read_roadmap(&map_text).map_err(spoken)?;
    let progress = read_progress(&text).map_err(spoken)?;
    refused(validate(&map, &[]))?;
    refused(tracked(&map, &progress))?;
    Ok((map, map_text, text))
}

pub fn one(
    map: &Roadmap,
    entry: &TopicEntry,
    done: &[Topic],
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
    refused(ordered(map, std::slice::from_ref(&topic)))?;
    let mut so_far = done.to_vec();
    so_far.push(topic.clone());
    refused(validate(map, &so_far))?;
    Ok((topic, text))
}

pub fn whole(map_text: &str, topics: &[Topic]) -> Result<(Roadmap, String), Refused> {
    let marked = generated(map_text);
    let map = read_roadmap(&marked).map_err(|error| Refused {
        why: vec![error.to_string()],
        topics: Vec::new(),
    })?;

    let mut found = validate(&map, topics);
    found.extend(ordered(&map, topics));
    if found.is_empty() {
        return Ok((map, marked));
    }
    Err(Refused {
        why: found.iter().map(ToString::to_string).collect(),
        topics: found
            .iter()
            .flat_map(Violation::topics)
            .map(str::to_owned)
            .collect(),
    })
}

fn block(answer: &str, schema: &str) -> Result<String, Vec<String>> {
    pick(answer, schema).ok_or_else(|| {
        vec![format!(
            "в ответе нет блока, объявляющего `schema: {schema}`"
        )]
    })
}

fn refused(found: Vec<Violation>) -> Result<(), Vec<String>> {
    if found.is_empty() {
        return Ok(());
    }
    Err(found.iter().map(ToString::to_string).collect())
}

fn spoken(error: impl ToString) -> Vec<String> {
    vec![error.to_string()]
}
