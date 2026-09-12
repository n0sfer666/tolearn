use tolearn_generate::Online;
use tolearn_generate::sources::CACHE;
use tolearn_generate::start::Kit;
use tolearn_offline::store::Store;
use tolearn_provider::Stop;

use super::clock::now;
use super::context::Context;
use super::error::IpcError;
use super::settings::stored;

pub fn kitted<T>(
    context: &Context,
    online: &Online<'_>,
    stop: &Stop,
    work: impl FnOnce(Kit<'_>) -> Result<T, IpcError>,
) -> Result<T, IpcError> {
    let cache = context.data().join(CACHE);
    let mut store = Store::open(&cache, stored(context)?.budget_bytes())
        .map_err(|error| IpcError::unwritable(&cache, &error.to_string()))?;
    let tools = context.tools();
    let source = tools.fetcher()?;
    let renderer = tools.rendering();
    let painter = tools.painting();
    let progress = tools.progress();
    work(Kit {
        online,
        source: source.as_ref(),
        renderer: renderer.as_ref(),
        store: &mut store,
        painter: painter.as_ref(),
        progress: progress.as_ref(),
        stop,
        data: context.data(),
        at: now(),
    })
}
