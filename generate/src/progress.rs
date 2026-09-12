use crate::error::GenerateError;
use crate::step::Step;

pub trait Progress {
    fn began(&self, step: Step);
    fn ended(&self, step: Step);
}

impl Progress for () {
    fn began(&self, _step: Step) {}
    fn ended(&self, _step: Step) {}
}

pub(crate) fn stepped<T>(
    progress: &dyn Progress,
    step: Step,
    work: impl FnOnce() -> Result<T, GenerateError>,
) -> Result<T, GenerateError> {
    progress.began(step);
    let done = work();
    progress.ended(step);
    done
}
