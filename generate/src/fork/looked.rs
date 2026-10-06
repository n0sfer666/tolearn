use std::path::Path;

use tolearn_core::library::Library;

use crate::error::GenerateError;

use super::after::After;
use super::ahead::{Ahead, Road, ahead};
use super::kept;
use super::offered::Fork;
use super::rules;

pub(super) enum Looked {
    Ready(Road, Fork),
    Open(Ahead),
}

pub(super) fn looked(data: &Path, after: &After<'_>) -> Result<Looked, GenerateError> {
    let tree = Library::at(data)
        .open(after.program)
        .map_err(GenerateError::Library)?;
    Ok(match ahead(tree, after)? {
        Road::Part(onward) => {
            let fork = onward.fork();
            Looked::Ready(Road::Part(onward), fork)
        }
        Road::Stage(ahead) => match kept::load(data, after)?.filter(|fork| fresh(fork, &ahead)) {
            Some(fork) => Looked::Ready(Road::Stage(ahead), fork),
            None => Looked::Open(ahead),
        },
    })
}

fn fresh(fork: &Fork, ahead: &Ahead) -> bool {
    fork.variants
        .first()
        .is_some_and(|variant| variant.row == ahead.next)
        && rules::check(fork, &ahead.leaf.map).is_empty()
}
