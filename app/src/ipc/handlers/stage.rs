use std::collections::BTreeMap;

use tolearn_core::block::Block;
use tolearn_core::library::Library;
use tolearn_core::program::Tree;
use tolearn_core::stage::{Check, Question};
use tolearn_core::state::{Attempt, State};

use crate::ipc::clarifying::views;
use crate::ipc::clock;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::picture;
use crate::ipc::reading::{AskView, BlockView, ClaimView, StageIn, StageOut, TaskView};
use crate::ipc::shelf;

pub fn run(context: &Context, input: &StageIn) -> Result<StageOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let stage = branch.tree.stages.get(&input.stage).ok_or_else(|| {
        IpcError::new(
            "stage.absent",
            format!("этапа `{}` нет или он ещё не сгенерирован", input.stage),
        )
    })?;
    let today = tolearn_generate::start::day(clock::now());
    let node = &branch.tree.program.uuid;
    let (ticks, workdir, last, drafts, clarifications) =
        State::update(context.data(), &tree.program.uuid, |state| {
            state.open(node, &stage.id, &today);
            (
                state.ticks(node, &stage.id),
                state.workdir.clone(),
                state.last_attempt(node, &stage.id).cloned(),
                state.drafts(node, &stage.id),
                views(state, node, &stage.id),
            )
        })?;
    let view = |block: &Block| viewed(&library, &tree, &branch.prefix, block);
    Ok(StageOut {
        program: tree.program.uuid.clone(),
        node: branch.tree.program.uuid.clone(),
        node_title: branch.tree.program.title.clone(),
        id: stage.id.clone(),
        title: stage.title.clone(),
        blocks: stage.blocks.iter().map(view).collect::<Result<_, _>>()?,
        practice: TaskView {
            task: stage
                .practice
                .task
                .iter()
                .map(view)
                .collect::<Result<_, _>>()?,
            deliverable: stage.practice.deliverable.clone(),
            constraints: stage.practice.constraints.iter().map(claim).collect(),
            acceptance: stage.practice.acceptance.iter().map(claim).collect(),
        },
        questions: stage
            .questions
            .iter()
            .map(|question| asked(question, last.as_ref(), &drafts))
            .collect(),
        ticks,
        workdir,
        clarifications,
    })
}

fn viewed(
    library: &Library,
    tree: &Tree,
    prefix: &str,
    block: &Block,
) -> Result<BlockView, IpcError> {
    let src = match &block.asset {
        Some(asset) => {
            let bytes = library.asset(tree, &format!("{prefix}{asset}"))?;
            Some(picture::uri(asset, &bytes))
        }
        None => None,
    };
    Ok(BlockView {
        id: block.id.clone(),
        kind: block.kind.label().to_owned(),
        text: block.text.clone(),
        lang: block.lang.clone(),
        src,
        license: block.license.clone(),
        attribution: block.attribution.clone(),
        source: block.source.clone(),
    })
}

fn asked(
    question: &Question,
    last: Option<&Attempt>,
    drafts: &BTreeMap<String, String>,
) -> AskView {
    let graded = last.and_then(|attempt| {
        attempt
            .per_question
            .iter()
            .find(|row| row.id == question.id)
    });
    AskView {
        id: question.id.clone(),
        text: question.text.clone(),
        result: graded.map(|row| row.result.label().to_owned()),
        missed: graded.map(|row| row.missed.clone()).unwrap_or_default(),
        draft: drafts.get(&question.id).cloned().unwrap_or_default(),
    }
}

fn claim(check: &Check) -> ClaimView {
    ClaimView {
        id: check.id.clone(),
        claim: check.claim.clone(),
        check: check.check.clone(),
        expect: check.expect.clone(),
    }
}
