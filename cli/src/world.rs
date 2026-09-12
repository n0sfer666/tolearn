use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tolearn_core::places::places;
use tolearn_generate::REACH_TIMEOUT_SECS;
use tolearn_offline::page::{Source, Web};
use tolearn_offline::reach::{Ping, Reach};
use tolearn_provider::{Keychain, Vault};

use crate::error::CliError;

const FETCH_SECS: u64 = 30;

pub struct World {
    pub reach: Box<dyn Reach>,
    pub source: Box<dyn Source>,
    pub vault: Box<dyn Vault>,
    pub config: PathBuf,
    pub data: PathBuf,
    pub log: Box<dyn Write>,
    pub at: i64,
}

impl World {
    pub fn real() -> Result<Self, CliError> {
        let home = std::env::home_dir()
            .ok_or_else(|| CliError::Setup("домашний каталог не найден".to_owned()))?;
        let reach = Ping::new(REACH_TIMEOUT_SECS).map_err(CliError::Setup)?;
        let source = Web::new(FETCH_SECS).map_err(CliError::Setup)?;
        let places = places(&home);
        Ok(Self {
            reach: Box::new(reach),
            source: Box::new(source),
            vault: Box::new(Keychain::app()),
            config: places.config,
            data: places.data,
            log: Box::new(std::io::stderr()),
            at: now(),
        })
    }
}

impl fmt::Debug for World {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("World")
            .field("reach", &self.reach)
            .field("config", &self.config)
            .field("data", &self.data)
            .field("at", &self.at)
            .finish_non_exhaustive()
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
        })
}
