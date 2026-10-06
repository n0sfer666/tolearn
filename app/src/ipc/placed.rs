use tolearn_generate::fork::After;

use super::context::Context;
use super::generating::GenerationWork;

pub fn placed(context: &Context, kind: &str, at: &After<'_>) -> GenerationWork {
    let work = GenerationWork::placed(kind, at.program, at.node, at.stage);
    match context.library().open(at.program) {
        Ok(tree) => GenerationWork {
            request: tree.program.generation.request.clone(),
            level: tree.program.level.clone(),
            locale: tree.program.generation.locale.clone(),
            ..work
        },
        Err(_) => work,
    }
}
