use monolith::core::{Options, init_client, retrieve_asset};
use url::Url;

use super::error::PageError;

pub trait Source {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError>;
}

#[derive(Debug, Clone, Copy)]
pub struct Web {
    pub timeout: u64,
}

impl Source for Web {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        let target = Url::parse(url)
            .map_err(|error| PageError::Unreachable(url.to_string(), error.to_string()))?;
        let options = Options {
            timeout: self.timeout,
            silent: true,
            ..Options::default()
        };
        let client = init_client(&options);
        let (bytes, _, _, _) = retrieve_asset(&mut None, &client, &target, &target, &options)
            .map_err(|error| PageError::Unreachable(url.to_string(), error.to_string()))?;
        Ok(bytes)
    }
}
