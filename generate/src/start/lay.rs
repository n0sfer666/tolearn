use std::fs;
use std::path::Path;

use tolearn_core::stage::Stage;

use crate::build::{Assets, described, staged};
use crate::error::GenerateError;

use super::lineage::Lineage;

const CHILDREN: &str = "children";

pub(super) fn lay(
    build: &Path,
    lineage: &Lineage,
    stage: &Stage,
    assets: &Assets,
) -> Result<(), GenerateError> {
    let _ = fs::remove_dir_all(build);
    let mut folder = build.to_path_buf();
    for (depth, program) in lineage.programs().enumerate() {
        if depth > 0 {
            folder = folder.join(CHILDREN).join(&program.uuid);
        }
        if program.uuid != lineage.leaf.uuid {
            described(&folder, program)?;
        }
    }
    staged(&folder, &lineage.leaf, stage, assets)
}
