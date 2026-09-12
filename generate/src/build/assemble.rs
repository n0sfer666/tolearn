use std::collections::BTreeMap;

use tolearn_core::block::{Block, Kind};
use tolearn_core::stage::Stage;

use crate::diagram::{self, Diagram};
use crate::error::GenerateError;
use crate::stage::Gathered;

use super::kit::Kit;
use crate::halt::checked;

pub(crate) type Assets = BTreeMap<String, Vec<u8>>;

pub(crate) fn assemble(
    kit: &Kit<'_>,
    mut stage: Stage,
    gathered: &Gathered,
) -> Result<(Stage, Assets), GenerateError> {
    let mut assets = Assets::new();
    for block in stage
        .blocks
        .iter_mut()
        .chain(stage.practice.task.iter_mut())
    {
        match block.kind {
            Kind::Diagram => {
                checked(kit.stop)?;
                let id = block.id.clone();
                *block = match diagram::draw(kit.painter, &block.text) {
                    Diagram::Drawn(drawn) => {
                        assets.insert(drawn.file, drawn.bytes);
                        Block { id, ..drawn.block }
                    }
                    Diagram::Source { block: code, .. } => Block { id, ..code },
                };
            }
            Kind::Image if block.asset.is_some() => {
                let found = gathered
                    .images
                    .iter()
                    .find(|image| image.block.asset == block.asset);
                if let Some(image) = found {
                    assets.insert(image.file.clone(), image.bytes.clone());
                }
            }
            _ => {}
        }
    }
    Ok((stage, assets))
}
