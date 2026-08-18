use std::time::Duration;

use reqwest::blocking::Client;

use crate::error::CheckError;

pub(crate) fn apart<T>(
    work: impl FnOnce() -> Result<T, CheckError> + Send + 'static,
) -> Result<T, CheckError>
where
    T: Send + 'static,
{
    std::thread::spawn(work)
        .join()
        .map_err(|_| CheckError::Unreachable("запрос оборвался".to_owned()))?
}

pub(crate) fn given(key: Option<&str>) -> Option<&str> {
    key.map(str::trim).filter(|key| !key.is_empty())
}

pub(crate) fn client(timeout: Duration) -> Result<Client, CheckError> {
    Client::builder().timeout(timeout).build().map_err(broken)
}

pub(crate) fn refused(status: u16) -> Option<CheckError> {
    if status == 401 || status == 403 {
        return Some(CheckError::Rejected);
    }
    if (200..300).contains(&status) {
        return None;
    }
    Some(CheckError::Answered(status))
}

pub(crate) fn broken(error: reqwest::Error) -> CheckError {
    CheckError::Unreachable(error.to_string())
}
