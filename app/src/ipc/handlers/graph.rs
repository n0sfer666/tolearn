use tolearn_core::Date;
use tolearn_core::graph::{Node, graph};
use tolearn_core::status::effective;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{GraphIn, GraphOut, NodeView};

pub fn run(_context: &Context, input: &GraphIn) -> Result<GraphOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    let drawn = graph(&opened.scan.roadmap, &opened.scan.topics, &statuses);

    Ok(GraphOut {
        nodes: drawn.nodes.iter().map(view).collect(),
    })
}

fn view(node: &Node) -> NodeView {
    NodeView {
        id: node.id.clone(),
        title: node.title.clone(),
        status: node.status.label().to_owned(),
        layer: u32::try_from(node.layer).unwrap_or(u32::MAX),
        depends_on: node.depends_on.clone(),
        blocked_by: node.blocked_by.clone(),
        unlocks: node.unlocks.clone(),
    }
}
