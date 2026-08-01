use monolith::core::{Options, retrieve_asset};
use reqwest::blocking::Client;
use url::Url;

use crate::net;

use super::error::PageError;

pub trait Source {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError>;
}

pub struct Web {
    client: Client,
    options: Options,
}

impl Web {
    pub fn new(timeout: u64) -> Result<Self, String> {
        let options = Options {
            timeout,
            silent: true,
            ..Options::default()
        };
        Ok(Self {
            client: net::client(timeout)?,
            options,
        })
    }

    pub fn client(&self) -> Client {
        self.client.clone()
    }
}

impl std::fmt::Debug for Web {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Web")
            .field("timeout", &self.options.timeout)
            .finish()
    }
}

impl Source for Web {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        let target = Url::parse(url)
            .map_err(|error| PageError::Unreachable(url.to_string(), error.to_string()))?;
        let (bytes, _, _, _) =
            retrieve_asset(&mut None, &self.client, &target, &target, &self.options)
                .map_err(|error| PageError::Unreachable(url.to_string(), error.to_string()))?;
        Ok(bytes)
    }
}
