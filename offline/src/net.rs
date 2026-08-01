use std::time::Duration;

use reqwest::blocking::Client;

pub(crate) fn client(timeout: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|error| error.to_string())
}
