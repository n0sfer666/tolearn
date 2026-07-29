use std::path::Path;

use tolearn_core::search::{Hit, Index, Refresh, SearchError, roadmap_id};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{HitView, SearchIn, SearchOut};
use crate::ipc::vaulted::{Store, broke, store};

const LIMIT: usize = 20;

pub fn run(context: &Context, input: &SearchIn) -> Result<SearchOut, IpcError> {
    let bundle = Path::new(&input.bundle);
    let roadmap = roadmap_id(bundle).map_err(failed)?;
    let kept = store(context, input.directory.as_ref())?;

    let (index, report) = match kept.sealed().is_some() {
        true => sealed(&kept, &roadmap, bundle)?,
        false => plain(context, &kept, &roadmap, bundle)?,
    };

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

fn plain(
    context: &Context,
    kept: &Store,
    roadmap: &str,
    bundle: &Path,
) -> Result<(Index, Refresh), IpcError> {
    let path = context.search(roadmap);
    let mut index = Index::read(&path).map_err(failed)?;
    let report = index.refresh(bundle, kept.root()).map_err(failed)?;
    if stale(&report) {
        index.save(&path).map_err(failed)?;
    }
    Ok((index, report))
}

fn sealed(kept: &Store, roadmap: &str, bundle: &Path) -> Result<(Index, Refresh), IpcError> {
    let store = kept.sealed().ok_or_else(missing)?;
    let name = format!("search-{roadmap}");
    let path = store.root().join(&name);
    let mut index = match store.blob(&name).map_err(broke)? {
        Some(text) => Index::parse(&text, &path).map_err(failed)?,
        None => Index::default(),
    };
    let report = index.refresh_kept(bundle, &kept.index()?).map_err(failed)?;
    if stale(&report) {
        store.keep(&name, &index.text()).map_err(broke)?;
    }
    Ok((index, report))
}

fn stale(report: &Refresh) -> bool {
    report.indexed > 0 || report.dropped > 0
}

fn missing() -> IpcError {
    IpcError::new(
        "search.unreadable",
        "зашифрованное хранилище конспектов не открыто".to_owned(),
    )
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
