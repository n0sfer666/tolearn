use tolearn_provider::Stop;

use crate::error::GenerateError;

pub(crate) fn checked(stop: &Stop) -> Result<(), GenerateError> {
    if stop.stopped() {
        Err(GenerateError::Cancelled)
    } else {
        Ok(())
    }
}

pub(crate) fn sealed(stop: &Stop) -> Result<(), GenerateError> {
    if stop.seal() {
        Ok(())
    } else {
        Err(GenerateError::Cancelled)
    }
}
