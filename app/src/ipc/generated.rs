use tolearn_generate::GenerateError;
use tolearn_provider::Stop;

use super::context::Context;
use super::error::IpcError;
use super::generating::GenerationWork;
use super::planning::refused;
use super::running::{Finale, Hold};

pub fn generated<T>(
    context: &Context,
    work: GenerationWork,
    run: impl FnOnce(&Stop) -> Result<T, IpcError>,
    ready: impl FnOnce(&T) -> Finale,
) -> Result<T, IpcError> {
    let claim = context.running().claim(Hold::generation(work))?;
    let result = run(claim.stop());
    if claim.stop().stopped() {
        claim.end(None);
        return Err(refused(GenerateError::Cancelled));
    }
    claim.end(match &result {
        Ok(done) => Some(ready(done)),
        Err(error) if error.code == GenerateError::Cancelled.code() => None,
        Err(error) => Some(Finale::Refused(error.clone())),
    });
    result
}
