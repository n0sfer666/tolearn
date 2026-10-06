use super::types::Stage;
use crate::block::{self, Block, Kind};
use crate::program::Violation;

pub fn check(stage: &Stage) -> Vec<Violation> {
    let blocks: Vec<&Block> = stage.every_block().collect();
    let expected = block::ids(blocks.iter().map(|block| block.text.as_str()));
    let mut found = Vec::new();
    for (block, expected) in blocks.into_iter().zip(expected) {
        if block.id != expected {
            found.push(Violation::BlockId {
                stage: stage.id.clone(),
                block: block.id.clone(),
                expected,
            });
        }
        fields(&stage.id, block, &mut found);
    }
    found
}

fn fields(stage: &str, block: &Block, found: &mut Vec<Violation>) {
    for (field, value) in block.extras() {
        if value.is_some() && !block.kind.fields().contains(&field) {
            found.push(Violation::ForeignBlockField {
                stage: stage.to_owned(),
                block: block.id.clone(),
                kind: block.kind,
                field,
            });
        }
    }
    if matches!(block.kind, Kind::Diagram | Kind::Image) && block.asset.is_none() {
        found.push(Violation::BlockWithoutAsset {
            stage: stage.to_owned(),
            block: block.id.clone(),
            kind: block.kind,
        });
    }
    if block.kind == Kind::Image && (block.license.is_none() || block.attribution.is_none()) {
        found.push(Violation::UnlicensedImage {
            stage: stage.to_owned(),
            block: block.id.clone(),
        });
    }
}
