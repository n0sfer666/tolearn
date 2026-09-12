use std::cell::RefCell;
use std::fmt;
use std::io::Write;
use std::path::Path;

use tolearn_core::settings::{Locale, Settings};
use tolearn_generate::sources::CACHE;
use tolearn_generate::start::Kit;
use tolearn_generate::{GenerateError, Online, Progress, online};
use tolearn_offline::page::{AsFetched, Source};
use tolearn_offline::reach::Reach;
use tolearn_offline::store::Store;

use super::provided::provided;
use super::speaker::Speaker;
use super::unpainted::Unpainted;
use crate::error::CliError;
use crate::world::World;

const SETTINGS: &str = "settings.yaml";

pub struct Crew {
    pub speaker: Speaker,
    pub painter: Unpainted,
    pub log: RefCell<Box<dyn Write>>,
    pub locale: Locale,
    pub at: i64,
    reach: Box<dyn Reach>,
    source: Box<dyn Source>,
    budget: u64,
}

impl Crew {
    pub fn gathered(world: World, provider: Option<&Path>) -> Result<Self, CliError> {
        let speaker = provided(&world.config, provider, world.vault.as_ref())?;
        let settings = world.config.join(SETTINGS);
        let settings = Settings::read(&settings)
            .map_err(|error| CliError::Setup(format!("`{}`: {error}", settings.display())))?;
        Ok(Self {
            speaker,
            painter: Unpainted::default(),
            log: RefCell::new(world.log),
            locale: settings.locale,
            at: world.at,
            reach: world.reach,
            source: world.source,
            budget: settings.budget_bytes(),
        })
    }

    pub fn online(&self) -> Result<Online<'_>, CliError> {
        Ok(online(self.reach.as_ref(), &self.speaker)?)
    }

    pub fn kitted<T>(
        &self,
        online: &Online<'_>,
        progress: &dyn Progress,
        data: &Path,
        work: impl FnOnce(Kit<'_>) -> Result<T, GenerateError>,
    ) -> Result<T, CliError> {
        let cache = data.join(CACHE);
        let mut store = Store::open(&cache, self.budget).map_err(|error| {
            CliError::Setup(format!("кэш `{}` не открыт: {error}", cache.display()))
        })?;
        let kit = Kit {
            online,
            source: self.source.as_ref(),
            renderer: &AsFetched,
            store: &mut store,
            painter: &self.painter,
            progress,
            stop: &self.speaker.stop,
            data,
            at: self.at,
        };
        Ok(work(kit)?)
    }
}

impl fmt::Debug for Crew {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("Crew")
            .field("speaker", &self.speaker)
            .field("painter", &self.painter)
            .field("at", &self.at)
            .finish_non_exhaustive()
    }
}
