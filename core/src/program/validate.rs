use std::collections::BTreeSet;

use super::MAX_DEPTH;
use super::check::check;
use super::tree::Tree;
use super::violation::Violation;
use crate::stage;

pub fn validate(tree: &Tree) -> Vec<Violation> {
    let mut found = Vec::new();
    walk(tree, 1, &mut BTreeSet::new(), &mut found);
    found
}

fn walk<'a>(
    tree: &'a Tree,
    depth: usize,
    seen: &mut BTreeSet<&'a str>,
    found: &mut Vec<Violation>,
) {
    let program = &tree.program;
    if depth >= MAX_DEPTH && !program.map.children.is_empty() {
        found.push(Violation::TooDeep {
            program: program.uuid.clone(),
        });
    }
    if !seen.insert(program.uuid.as_str()) {
        found.push(Violation::DuplicateUuid {
            uuid: program.uuid.clone(),
        });
    }
    found.extend(check(program));
    stages(tree, found);
    for (row, child) in &tree.children {
        if child.program.uuid != *row {
            found.push(Violation::ChildUuidMismatch {
                row: row.clone(),
                found: child.program.uuid.clone(),
            });
        }
        walk(child, depth + 1, seen, found);
    }
}

fn stages(tree: &Tree, found: &mut Vec<Violation>) {
    for (row, stage) in &tree.stages {
        if stage.id != *row {
            found.push(Violation::StageIdMismatch {
                row: row.clone(),
                found: stage.id.clone(),
            });
        }
        found.extend(stage::check(stage));
        for block in stage.every_block() {
            let missing = block
                .asset
                .as_ref()
                .filter(|asset| !tree.assets.contains(*asset));
            if let Some(asset) = missing {
                found.push(Violation::MissingAsset {
                    stage: row.clone(),
                    block: block.id.clone(),
                    asset: asset.clone(),
                });
            }
        }
    }
}
