use tolearn_core::Date;
use tolearn_core::queue::due;
use tolearn_core::registry::{Listed, Registry};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{DueView, QueueIn, QueueOut};

pub fn run(context: &Context, input: &QueueIn) -> Result<QueueOut, IpcError> {
    let today = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let registry = Registry::read(&context.registry())?;
    Ok(QueueOut {
        due: registry
            .entries()
            .iter()
            .flat_map(|listed| waiting(listed, today))
            .collect(),
    })
}

fn waiting(listed: &Listed, today: Date) -> Vec<DueView> {
    let path = listed.program.path.display().to_string();
    let Ok(opened) = open::open(&path) else {
        return Vec::new();
    };
    due(&opened.scan.topics, opened.document.progress(), today)
        .into_iter()
        .map(|item| DueView {
            program: listed.program.id.clone(),
            title: listed.program.title.clone(),
            bundle: path.clone(),
            topic: item.topic,
            topic_title: item.title,
            due: item.due,
            overdue: item.overdue,
        })
        .collect()
}
