use tolearn_core::state::{State, Turn, excerpt};
use tolearn_generate::clarify::{self, Doubt, FRAGMENT_CHARS};
use tolearn_generate::stage::Place;
use tolearn_generate::{Step, local, online, stepped};

use crate::ipc::clarified::{ClarificationsOut, ClarifyIn};
use crate::ipc::clarifying::{index, lost, unclear, views};
use crate::ipc::clock::now;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::examining::staged;
use crate::ipc::planning::{refused, voiced};
use crate::ipc::shelf;

const KIND: &str = "Уточнение";

fn said(text: Option<&str>) -> Option<String> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn picked(text: Option<&str>) -> Option<String> {
    said(text).map(|text| text.chars().take(FRAGMENT_CHARS).collect())
}

pub fn run(context: &Context, input: &ClarifyIn) -> Result<ClarificationsOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = staged(branch.tree, &input.stage)?;
    let block = unclear(stage, &input.block)?;
    let node = &branch.tree.program.uuid;
    let followed = match input.chain {
        Some(chain) => Some((chain, index(chain)?)),
        None => None,
    };
    let (chain, fragment) = match followed {
        Some((chain, at)) => State::read(context.data(), &tree.program.uuid)?
            .chain(node, &stage.id, at)
            .filter(|found| found.block == block.id && !found.clear)
            .map(|found| (found.turns.clone(), found.fragment.clone()))
            .ok_or_else(|| lost(chain))?,
        None => (Vec::new(), picked(input.fragment.as_deref())),
    };
    let question = said(Some(&input.question));
    let claim = context.running().claim(Some(&input.program))?;
    let model = voiced(context, KIND, claim.stop().clone())?;
    let online = if model.remote() {
        let reach = context.reach()?;
        online(reach.as_ref(), &model).map_err(refused)?
    } else {
        local(&model)
    };
    let place = Place::find(&branch.tree.program, &stage.id).ok_or_else(|| {
        IpcError::new(
            "stage.absent",
            format!("этапа `{}` нет на карте программы", stage.id),
        )
    })?;
    let doubt = Doubt {
        program: &tree.program.uuid,
        node,
        place,
        block,
        fragment: fragment.as_deref(),
        chain: &chain,
        question: question.as_deref(),
    };
    let at = now();
    let progress = context.tools().progress();
    let answer = stepped(progress.as_ref(), Step::Clarify, || {
        clarify::clarify(&online, context.data(), &doubt, at)
    })
    .map_err(refused)?;
    let turn = Turn {
        asked: question,
        answer,
    };
    let clarifications = State::update(context.data(), &tree.program.uuid, |state| {
        match followed.and_then(|(_, at)| state.chain_mut(node, &stage.id, at)) {
            Some(found) => found.turns.push(turn),
            None => {
                state.clarify(
                    node,
                    &stage.id,
                    &block.id,
                    &excerpt(&block.text),
                    fragment.as_deref(),
                    turn,
                );
            }
        }
        views(state, node, &stage.id)
    })?;
    Ok(ClarificationsOut { clarifications })
}
