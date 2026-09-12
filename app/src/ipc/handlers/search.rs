use tolearn_core::search::{Hit, Index, Refresh, SearchError};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{HitView, SearchIn, SearchOut};

const LIMIT: usize = 20;

pub fn run(context: &Context, input: &SearchIn) -> Result<SearchOut, IpcError> {
    let path = context.search();
    let mut index = Index::read(&path);
    let report = index.refresh(&context.library()).map_err(failed)?;
    if stale(&report) {
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

fn stale(report: &Refresh) -> bool {
    report.indexed > 0 || report.dropped > 0
}

fn view(hit: &Hit) -> HitView {
    HitView {
        kind: hit.kind.label().to_owned(),
        program: hit.program.clone(),
        node: hit.node.clone(),
        node_title: hit.node_title.clone(),
        stage: hit.stage.clone(),
        title: hit.title.clone(),
        block: hit.block.clone(),
        snippet: hit.snippet.clone(),
    }
}

fn failed(error: SearchError) -> IpcError {
    let code = match error {
        SearchError::Unwritable { .. } => "search.unwritable",
        SearchError::Unreadable { .. } | SearchError::Malformed { .. } => "search.unreadable",
    };
    IpcError::new(code, error.to_string())
}
