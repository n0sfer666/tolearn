use tolearn_core::program::Tree;
use tolearn_core::stage::Check;

use super::error::IpcError;

pub fn claim<'a>(tree: &'a Tree, stage: &str, claim: &str) -> Result<&'a Check, IpcError> {
    let found = tree.stages.get(stage).ok_or_else(|| {
        IpcError::new(
            "stage.absent",
            format!("этапа `{stage}` нет или он ещё не сгенерирован"),
        )
    })?;
    found
        .practice
        .constraints
        .iter()
        .chain(&found.practice.acceptance)
        .find(|check| check.id == claim)
        .ok_or_else(|| {
            IpcError::new(
                "claim.absent",
                format!("у практики этапа `{stage}` нет пункта `{claim}`"),
            )
        })
}
