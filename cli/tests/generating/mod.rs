#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]
#![allow(dead_code, reason = "опоры нужны не каждому тест-бинарнику")]

#[path = "../../../tests-support/scratch.rs"]
mod rooms;

pub mod answers;
pub mod locked;
pub mod log;
pub mod net;
pub mod speaking;
pub mod web;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use tolearn_cli::{CliError, World};
use tolearn_core::library::Library;
use tolearn_core::program::Tree;
use tolearn_provider::{Http, Provider, Remembered, Vault};

use self::log::Log;
use locked::Locked;
use net::Net;
use speaking::Speaking;
use web::Web;

pub const REQUEST: &str = "Хочу писать чиптюн";
pub const LEVEL: &str = "Нот не знаю";
const AT: i64 = 1_757_750_400;

static DESKS: AtomicUsize = AtomicUsize::new(0);

pub struct Desk {
    pub root: PathBuf,
    pub config: PathBuf,
    pub out: PathBuf,
    pub log: Log,
    pub up: bool,
    pub locked: bool,
}

impl Desk {
    pub fn new(model: &Speaking) -> Self {
        let root = rooms::named(&format!(
            "cli-generating-{}",
            DESKS.fetch_add(1, Ordering::Relaxed)
        ));
        let config = root.join("config");
        std::fs::create_dir_all(&config).unwrap();
        provided(&model.endpoint)
            .save(&config.join("provider.yaml"))
            .unwrap();
        Self {
            out: root.join("out"),
            root,
            config,
            log: Log::default(),
            up: true,
            locked: false,
        }
    }

    pub fn run(&self, args: &[&str]) -> Result<String, CliError> {
        let argv: Vec<String> = args.iter().map(|word| (*word).to_owned()).collect();
        tolearn_cli::run(&argv, || Ok(self.world()))
    }

    pub fn world(&self) -> World {
        let vault: Box<dyn Vault> = match self.locked {
            true => Box::new(Locked),
            false => Box::new(Remembered::default()),
        };
        World {
            reach: Box::new(Net(self.up)),
            source: Box::new(Web),
            vault,
            config: self.config.clone(),
            data: self.data(),
            log: Box::new(self.log.clone()),
            at: AT,
        }
    }

    pub fn data(&self) -> PathBuf {
        self.root.join("data")
    }

    pub fn out(&self) -> &str {
        self.out.to_str().unwrap()
    }

    pub fn begun(&self) -> String {
        self.run(&["new", REQUEST, "--level", LEVEL, "--out", self.out()])
            .unwrap()
    }

    pub fn program(&self) -> Tree {
        let entries = Library::at(&self.out).list().unwrap();
        assert_eq!(entries.len(), 1, "{entries:?}");
        entries[0].program.clone().unwrap()
    }

    pub fn path(&self) -> PathBuf {
        self.out.join("programs").join(self.program().program.uuid)
    }
}

impl Drop for Desk {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub fn provided(endpoint: &str) -> Provider {
    Provider {
        enabled: true,
        local: Http {
            endpoint: endpoint.to_owned(),
            model: "llama3:8b".to_owned(),
            ..Http::local()
        },
        ..Provider::default()
    }
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}
