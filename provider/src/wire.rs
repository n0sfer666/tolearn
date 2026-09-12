use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use reqwest::blocking::Client;
use tolearn_runner::Stop;

use crate::error::CheckError;

const POLL: Duration = Duration::from_millis(50);

pub(crate) fn apart<T>(
    work: impl FnOnce() -> Result<T, CheckError> + Send + 'static,
    stop: &Stop,
) -> Result<T, CheckError>
where
    T: Send + 'static,
{
    let (sent, heard) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sent.send(work());
    });
    loop {
        match heard.recv_timeout(POLL) {
            Ok(result) => return result,
            Err(RecvTimeoutError::Timeout) if stop.stopped() => {
                return Err(CheckError::Cancelled);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err(CheckError::Unreachable("запрос оборвался".to_owned()));
            }
        }
    }
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
