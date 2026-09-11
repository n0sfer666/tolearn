use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::reading::{CrumbView, NodeIn, NodeOut};
use crate::ipc::shelf;

pub fn run(context: &Context, input: &NodeIn) -> Result<NodeOut, IpcError> {
    let tree = context.library().open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let program = &branch.tree.program;
    Ok(NodeOut {
        program: tree.program.uuid.clone(),
        uuid: program.uuid.clone(),
        title: program.title.clone(),
        goal: program.goal.clone(),
        level: program.level.clone(),
        hours: shelf::span(program.map.hours()),
        trail: branch
            .trail
            .iter()
            .map(|node| CrumbView {
                uuid: node.program.uuid.clone(),
                title: node.program.title.clone(),
            })
            .collect(),
        stages: shelf::stages(branch.tree),
        children: shelf::children(branch.tree),
    })
}
