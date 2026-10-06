use tolearn_core::block::{Block, Kind};
use tolearn_core::stage::Stage;
use tolearn_core::state::State;

use super::clarified::{ChainIn, ClarificationView, ClarificationsOut, TurnView};
use super::context::Context;
use super::error::IpcError;
use super::examining::staged;
use super::shelf;

pub fn unclear<'a>(stage: &'a Stage, id: &str) -> Result<&'a Block, IpcError> {
    stage
        .blocks
        .iter()
        .find(|block| block.id == id && block.kind != Kind::Heading)
        .ok_or_else(|| {
            IpcError::new(
                "clarification.block",
                format!(
                    "в теории этапа `{}` нет блока `{id}`, который можно уточнить",
                    stage.id
                ),
            )
        })
}

pub fn views(state: &State, node: &str, stage: &str) -> Vec<ClarificationView> {
    state
        .clarifications_of(node, stage)
        .into_iter()
        .filter_map(|(index, chain)| {
            Some(ClarificationView {
                chain: u32::try_from(index).ok()?,
                block: chain.block.clone(),
                excerpt: chain.excerpt.clone(),
                fragment: chain.fragment.clone(),
                turns: chain
                    .turns
                    .iter()
                    .map(|turn| TurnView {
                        asked: turn.asked.clone(),
                        answer: turn.answer.clone(),
                    })
                    .collect(),
                clear: chain.clear,
            })
        })
        .collect()
}

pub fn lost(chain: u32) -> IpcError {
    IpcError::new(
        "clarification.absent",
        format!("уточнения №{chain} у этого этапа нет"),
    )
}

pub fn index(chain: u32) -> Result<usize, IpcError> {
    usize::try_from(chain).map_err(|_| lost(chain))
}

pub fn chained(
    context: &Context,
    input: &ChainIn,
    change: impl FnOnce(&mut State, &str, &str, usize) -> bool,
) -> Result<ClarificationsOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let at = index(input.chain)?;
    let node = &branch.tree.program.uuid;
    let changed = State::update(context.data(), &tree.program.uuid, |state| {
        change(state, node, &stage.id, at).then(|| views(state, node, &stage.id))
    })?;
    changed
        .map(|clarifications| ClarificationsOut { clarifications })
        .ok_or_else(|| lost(input.chain))
}
