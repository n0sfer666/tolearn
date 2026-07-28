use std::path::Path;

use tolearn_core::search::{Hit, Index, SearchError, roadmap_id};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::notes::root;
use crate::ipc::types::{HitView, SearchIn, SearchOut};

const LIMIT: usize = 20;

pub fn run(context: &Context, input: &SearchIn) -> Result<SearchOut, IpcError> {
    let bundle = Path::new(&input.bundle);
    let roadmap = roadmap_id(bundle).map_err(failed)?;
    let path = context.search(&roadmap);

    let mut index = Index::read(&path).map_err(failed)?;
    let report = index
        .refresh(bundle, &root(context, input.directory.as_ref())?)
        .map_err(failed)?;
    if report.indexed > 0 || report.dropped > 0 {
        index.save(&path).map_err(failed)?;
    }

    let limit = usize::try_from(input.limit).unwrap_or(LIMIT);
    Ok(SearchOut {
        hits: index
            .find(&input.query, if limit == 0 { LIMIT } else { limit })
            .iter()
            .map(view)
            .collect(),
        indexed: u32::try_from(report.indexed).unwrap_or(u32::MAX),
    })
}

fn view(hit: &Hit) -> HitView {
    HitView {
        kind: hit.kind.label().to_owned(),
        roadmap: hit.roadmap.clone(),
        topic: hit.topic.clone(),
        title: hit.title.clone(),
        snippet: hit.snippet.clone(),
    }
}

fn failed(error: SearchError) -> IpcError {
    let code = match error {
        SearchError::Unwritable { .. } => "search.unwritable",
        SearchError::Bundle(_) => "search.bundle",
        SearchError::Unreadable { .. } | SearchError::Malformed { .. } => "search.unreadable",
    };
    IpcError::new(code, error.to_string())
}
