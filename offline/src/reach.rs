use std::error::Error;

use reqwest::blocking::Client;

use crate::net;

pub trait Reach {
    fn reach(&self, url: &str) -> Result<(), String>;
}

#[derive(Debug)]
pub struct Ping {
    client: Client,
}

impl Ping {
    pub fn new(timeout: u64) -> Result<Self, String> {
        Ok(Self {
            client: net::client(timeout)?,
        })
    }
}

impl Reach for Ping {
    fn reach(&self, url: &str) -> Result<(), String> {
        self.client
            .head(url)
            .send()
            .map(drop)
            .map_err(|error| reason(&error))
    }
}

fn reason(error: &reqwest::Error) -> String {
    let mut cause: &dyn Error = error;
    while let Some(source) = cause.source() {
        cause = source;
    }
    cause.to_string()
}
