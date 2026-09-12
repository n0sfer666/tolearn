use tolearn_provider::Stop;

use crate::error::GenerateError;
use crate::halt::checked;
use crate::progress::{Progress, stepped};
use crate::step::Step;

pub(super) fn guarded<T>(
    progress: &dyn Progress,
    stop: &Stop,
    step: Step,
    work: impl FnOnce() -> Result<T, GenerateError>,
) -> Result<T, GenerateError> {
    stepped(progress, step, || {
        checked(stop)?;
        work()
    })
}
