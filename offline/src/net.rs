use std::time::Duration;

use reqwest::blocking::Client;

pub(crate) const USER_AGENT: &str = concat!("tolearn/", env!("CARGO_PKG_VERSION"));

pub(crate) fn client(timeout: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| error.to_string())
}
