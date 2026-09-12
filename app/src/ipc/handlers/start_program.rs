use std::time::{SystemTime, UNIX_EPOCH};

use tolearn_generate::online;
use tolearn_generate::sources::CACHE;
use tolearn_generate::start::{self, Kit};
use tolearn_offline::store::Store;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::planning::{asked, refused, taken, told, voiced};
use crate::ipc::settings::stored;
use crate::ipc::started::{StartProgramIn, StartProgramOut};

const KIND: &str = "Этап программы";

pub fn run(context: &Context, input: &StartProgramIn) -> Result<StartProgramOut, IpcError> {
    told("уровень", &input.level)?;
    let request = asked(context, &input.request, &input.level)?;
    let plan = taken(&input.plan)?;
    let claim = context.running().claim()?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let reach = context.reach()?;
    let online = online(reach.as_ref(), &model).map_err(refused)?;
    let cache = context.data().join(CACHE);
    let mut store = Store::open(&cache, stored(context)?.budget_bytes())
        .map_err(|error| IpcError::unwritable(&cache, &error.to_string()))?;
    let tools = context.tools();
    let source = tools.fetcher()?;
    let renderer = tools.rendering();
    let painter = tools.painting();
    let progress = tools.progress();
    let kit = Kit {
        online: &online,
        source: source.as_ref(),
        renderer: renderer.as_ref(),
        store: &mut store,
        painter: painter.as_ref(),
        progress: progress.as_ref(),
        stop: claim.stop(),
        data: context.data(),
        at: now(),
    };
    let program = start::start(kit, &request, &plan).map_err(refused)?;
    Ok(StartProgramOut { program })
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
        })
}
