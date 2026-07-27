use tolearn_core::bundle::validate;

use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{ValidateIn, ValidateOut, Violation};

pub fn run(input: &ValidateIn) -> Result<ValidateOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let violations = validate(&scan.roadmap, &scan.topics);
    Ok(ValidateOut {
        ok: violations.is_empty(),
        violations: violations
            .iter()
            .map(|violation| Violation {
                code: violation.code().to_owned(),
                message: violation.to_string(),
            })
            .collect(),
    })
}
