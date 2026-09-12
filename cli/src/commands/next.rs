use std::time::Instant;

use tolearn_generate::fork::{self, After, NextError};
use tolearn_generate::{GenerateError, Step, stepped};

use super::offered::offered;
use crate::args::Next;
use crate::error::CliError;
use crate::out::Output;
use crate::session::{Built, Crew, Herald, apart, only, opened, position, row, unsettled};
use crate::world::World;

pub fn run(next: &Next, world: World) -> Result<Output, CliError> {
    apart(&next.data, &world.data)?;
    let tree = only(&next.data)?;
    let at = position(&tree).ok_or_else(|| {
        CliError::Data(format!(
            "в `{}` нет созданного этапа: развилка открывается после него",
            next.data.display()
        ))
    })?;
    let after = After {
        program: &tree.program.uuid,
        node: &at.node.program.uuid,
        stage: &at.row.id,
    };
    match next.choice {
        None => offer(next, world, &after),
        Some(choice) => take(next, world, &after, choice),
    }
}

fn offer(next: &Next, world: World, after: &After<'_>) -> Result<Output, CliError> {
    if let Some(fork) = fork::known(&next.data, after)? {
        return Ok(offered(&next.data, after, &fork));
    }
    let crew = Crew::gathered(world, next.provider.as_deref())?;
    let online = crew.online()?;
    let herald = Herald::new(&crew);
    let fork = stepped(&herald, Step::Fork, || {
        fork::propose(&online, &next.data, after, crew.at)
    })?;
    Ok(offered(&next.data, after, &fork))
}

fn take(next: &Next, world: World, after: &After<'_>, choice: usize) -> Result<Output, CliError> {
    let index = choice.saturating_sub(1);
    let count = fork::known(&next.data, after)?
        .ok_or_else(|| GenerateError::from(NextError::Unforked(after.stage.to_owned())))?
        .variants
        .len();
    if index >= count {
        return Err(GenerateError::from(NextError::Choice {
            choice: index,
            count,
        })
        .into());
    }
    let began = Instant::now();
    let crew = Crew::gathered(world, next.provider.as_deref())?;
    let online = crew.online()?;
    let herald = Herald::new(&crew);
    let stage = crew.kitted(&online, &herald, &next.data, |kit| {
        fork::take(kit, after, index)
    })?;
    let tree = opened(&next.data, after.program)?;
    let made = row(&tree, after.node, &stage).ok_or_else(|| {
        unsettled(
            &next.data,
            after.program,
            format!("этапа `{stage}` нет в карте"),
        )
    })?;
    Ok(Built {
        data: &next.data,
        program: after.program,
        node: after.node,
        stage: &stage,
        title: &made.title,
        elapsed: began.elapsed(),
        spent: crew.speaker.meter.spent(),
        mermaid: crew.painter.count(),
    }
    .shown())
}
