use tolearn_core::link::{Link, LinkError};
use tolearn_core::registry::Registry;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{FollowIn, FollowOut};

pub fn run(context: &Context, input: &FollowIn) -> Result<FollowOut, IpcError> {
    let link = tolearn_core::link::parse(&input.url).map_err(refused)?;
    let bundle = program(context, &link)?;
    let scan = open::read(&bundle)?;
    if !scan.topics.iter().any(|topic| topic.id == link.topic) {
        return Err(IpcError::new(
            "link.unknown-topic",
            format!("в программе `{}` нет темы `{}`", link.roadmap, link.topic),
        ));
    }
    Ok(FollowOut {
        program: bundle,
        topic: link.topic,
    })
}

fn program(context: &Context, link: &Link) -> Result<String, IpcError> {
    let registry = Registry::read(&context.registry())?;
    let entries = registry.entries();
    let listed = entries
        .iter()
        .find(|listed| listed.program.id == link.roadmap)
        .ok_or_else(|| {
            IpcError::new(
                "link.unknown-program",
                format!("программа `{}` не в реестре", link.roadmap),
            )
        })?;
    if !listed.reachable {
        return Err(IpcError::new(
            "link.unreachable",
            format!("программа `{}` недоступна по своему пути", link.roadmap),
        ));
    }
    Ok(listed.program.path.display().to_string())
}

fn refused(error: LinkError) -> IpcError {
    IpcError::new(error.code(), error.to_string())
}
