use std::path::Path;
use std::time::Duration;

use tolearn_core::state::State;
use tolearn_runner::{Limits, Outcome, Run, RunError};

use super::workdir::absent;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::practice::claim;
use crate::ipc::practiced::{CheckClaimIn, CheckClaimOut};
use crate::ipc::shelf;

const LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    silence: None,
    output_bytes: 64 * 1024,
};

pub fn run(context: &Context, input: &CheckClaimIn) -> Result<CheckClaimOut, IpcError> {
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let check = claim(branch.tree, &input.stage, &input.claim)?;
    let command = check.check.as_deref().ok_or_else(|| {
        IpcError::new(
            "claim.unchecked",
            format!("у пункта `{}` нет команды проверки", check.id),
        )
    })?;
    let workdir = State::read(context.data(), &tree.program.uuid)?
        .workdir
        .ok_or_else(|| {
            IpcError::new(
                "workdir.unset",
                "папка практики не выбрана — выберите её, прежде чем запускать".to_owned(),
            )
        })?;
    let ran =
        tolearn_runner::run(command, Path::new(&workdir), LIMITS).map_err(|error| match error {
            RunError::NoDirectory(_) => absent(&workdir),
            other => IpcError::new("check.failed", other.to_string()),
        })?;
    Ok(viewed(ran))
}

fn viewed(ran: Run) -> CheckClaimOut {
    let (outcome, code) = match ran.outcome {
        Outcome::Finished { code } => ("finished", code),
        Outcome::TimedOut => ("timeout", None),
        Outcome::WentQuiet => ("silence", None),
        Outcome::Stopped => ("stopped", None),
    };
    CheckClaimOut {
        outcome: outcome.to_owned(),
        code,
        stdout: ran.stdout,
        stderr: ran.stderr,
        truncated: ran.truncated,
    }
}
