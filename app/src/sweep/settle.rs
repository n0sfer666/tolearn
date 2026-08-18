use tolearn_core::Date;
use tolearn_core::progress::save;
use tolearn_core::protocol::{apply, record};
use tolearn_core::status::effective;

use crate::ipc::open::{self, Opened};
use crate::ipc::verdict::{of, read};
use crate::ipc::{Context, IpcError};

use super::run::Run;
use super::store;

#[derive(Debug)]
pub struct Settled {
    pub topic: String,
    pub title: String,
    pub result: String,
    pub status: String,
}

pub fn accept(
    context: &Context,
    opened: &mut Opened,
    run: &Run,
    day: Date,
) -> Result<Vec<Settled>, IpcError> {
    let mut settled: Vec<(String, String, String)> = Vec::new();
    for leg in &run.legs {
        let Some(text) = leg.verdict.as_deref() else {
            continue;
        };
        let topic = read(opened, &leg.topic)?;
        let parsed = of(text, topic)?;
        let state = opened.document.progress().state(&leg.topic).cloned();
        let applied = apply(
            &parsed,
            topic,
            state.as_ref(),
            &format!("{day}T00:00:00Z"),
            day,
        );
        record(&mut opened.document, &leg.topic, &applied)?;
        settled.push((
            leg.topic.clone(),
            leg.title.clone(),
            parsed.result.label().to_owned(),
        ));
    }
    if settled.is_empty() {
        return Err(IpcError::new(
            "sweep.unfinished",
            "вердиктов ещё нет — прогон не завершён".to_owned(),
        ));
    }
    let file = open::progress_file(&opened.scan);
    save(&file, &opened.document)
        .map_err(|error| IpcError::unwritable(&file, &error.to_string()))?;
    store::forget(context, &opened.scan.roadmap.id);

    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    Ok(settled
        .into_iter()
        .map(|(topic, title, result)| Settled {
            status: statuses
                .get(topic.as_str())
                .map(|status| status.label().to_owned())
                .unwrap_or_default(),
            topic,
            title,
            result,
        })
        .collect())
}
