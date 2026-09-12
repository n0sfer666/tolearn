use std::time::Instant;

use tolearn_generate::plan::{self, Request};
use tolearn_generate::{Step, start, stepped};

use crate::args::New;
use crate::error::CliError;
use crate::out::Output;
use crate::session::{Built, Crew, Herald, apart, empty, opened, position, unsettled};
use crate::world::World;

pub fn run(new: &New, world: World) -> Result<Output, CliError> {
    apart(&new.out, &world.data)?;
    empty(&new.out)?;
    let began = Instant::now();
    let crew = Crew::gathered(world, new.provider.as_deref())?;
    let request = Request {
        request: new.request.clone(),
        level: new.level.clone(),
        locale: crew.locale.label().to_owned(),
    };
    let online = crew.online()?;
    let herald = Herald::new(&crew);
    let plan = stepped(&herald, Step::Plan, || plan::plan(&online, &request))?;
    let uuid = crew.kitted(&online, &herald, &new.out, |kit| {
        start::start(kit, &request, &plan)
    })?;
    let tree = opened(&new.out, &uuid)?;
    let at = position(&tree).ok_or_else(|| unsettled(&new.out, &uuid, "созданного этапа нет"))?;
    Ok(Built {
        data: &new.out,
        program: &tree.program.uuid,
        node: &at.node.program.uuid,
        stage: &at.row.id,
        title: &at.row.title,
        elapsed: began.elapsed(),
        spent: crew.speaker.meter.spent(),
        mermaid: crew.painter.count(),
    }
    .shown())
}
