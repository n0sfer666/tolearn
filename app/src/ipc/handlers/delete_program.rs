use tolearn_core::library::Library;
use tolearn_generate::sources::CACHE;
use tolearn_offline::store::Store;

use crate::discard;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::reading::{DeleteProgramIn, DeleteProgramOut};
use crate::ipc::settings::stored;

pub fn run(context: &Context, input: &DeleteProgramIn) -> Result<DeleteProgramOut, IpcError> {
    let library = context.library();
    let tree = match library.open(&input.program) {
        Ok(tree) => tree,
        Err(absent) => return Err(inner(&library, &input.program).unwrap_or_else(|| absent.into())),
    };
    let _erasing = context.running().erasing(&input.program)?;
    let uuids: Vec<String> = tree.uuids().into_iter().collect();
    checked(&library, &input.program, &uuids)?;
    released(context, &uuids).map_err(unbound)?;
    discard::discard(context.bin(), context.data(), &input.program, &uuids).map_err(left)?;
    Ok(DeleteProgramOut {
        title: tree.program.title.clone(),
    })
}

fn inner(library: &Library, uuid: &str) -> Option<IpcError> {
    let entries = library.list().ok()?;
    let held = entries
        .into_iter()
        .filter_map(|entry| entry.program.ok())
        .find(|tree| tree.program.uuid != uuid && tree.uuids().contains(uuid))?;
    Some(IpcError::new(
        "delete.subprogram",
        format!(
            "`{uuid}` — подпрограмма внутри «{}»: удалить можно только программу целиком",
            held.program.title
        ),
    ))
}

fn checked(library: &Library, root: &str, uuids: &[String]) -> Result<(), IpcError> {
    if !discard::plain(root) || !uuids.iter().all(|uuid| discard::plain(uuid)) {
        return Err(broken(format!("в дереве `{root}` нечитаемый uuid")));
    }
    match strange(library, root, uuids) {
        Some(shared) => Err(broken(format!(
            "`{shared}` числится ещё за одной программой библиотеки"
        ))),
        None => Ok(()),
    }
}

fn strange(library: &Library, root: &str, uuids: &[String]) -> Option<String> {
    let entries = library.list().ok()?;
    entries
        .into_iter()
        .filter_map(|entry| entry.program.ok())
        .filter(|tree| tree.program.uuid != root)
        .find_map(|tree| {
            let held = tree.uuids();
            uuids
                .iter()
                .find(|uuid| held.contains(uuid.as_str()))
                .map(ToString::to_string)
        })
}

fn released(context: &Context, uuids: &[String]) -> Result<(), String> {
    let settings = stored(context).map_err(|refusal| refusal.message)?;
    let mut store = Store::open(&context.data().join(CACHE), settings.budget_bytes())
        .map_err(|error| error.to_string())?;
    for uuid in uuids {
        store.release(uuid).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn broken(reason: String) -> IpcError {
    IpcError::new("delete.broken", format!("{reason}: удаление отменено"))
}

fn unbound(reason: String) -> IpcError {
    IpcError::new(
        "delete.cache",
        format!("привязки офлайн-кэша не снялись: {reason}"),
    )
}

fn left(rooms: Vec<String>) -> IpcError {
    IpcError::new(
        "delete.left",
        format!("корзина не приняла: {}", rooms.join(", ")),
    )
}
