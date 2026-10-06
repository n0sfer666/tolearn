use tolearn_core::state::State;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::reading::{BookView, CrumbView, NodeIn, NodeOut, PageView, SourcesView};
use crate::ipc::shelf;

pub fn run(context: &Context, input: &NodeIn) -> Result<NodeOut, IpcError> {
    let tree = context.library().open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let state = State::read(context.data(), &tree.program.uuid)?;
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
        stages: shelf::stages(branch.tree, &state),
        children: shelf::children(branch.tree, &state),
        summary: shelf::summary(branch.tree, &state),
        sources: SourcesView {
            books: program
                .sources
                .books
                .iter()
                .map(|book| BookView {
                    title: book.title.clone(),
                    authors: book.authors.clone(),
                    chapter: book.chapter.clone(),
                })
                .collect(),
            pages: program
                .sources
                .pages
                .iter()
                .map(|page| PageView {
                    title: page.title.clone(),
                    url: page.url.clone(),
                    checked_at: page.checked_at.clone(),
                })
                .collect(),
        },
    })
}
