use std::cell::RefCell;
use std::fmt::Display;
use std::path::PathBuf;

use tolearn_core::topic::Material;
use tolearn_offline::mirror::{self, Limits};
use tolearn_offline::page::{self, Fetching, Source, Web};
use tolearn_offline::queue::{Saver, Strategy};
use tolearn_offline::repo;
use tolearn_offline::store::{Fetched, Store};
use tolearn_offline::video::{self, Tools, Wanted};

use super::jobs::Job;
use super::renderer;

const TIMEOUT: u64 = 30;
const VIDEO: &str = "video.mp4";

pub struct Bundled<'a> {
    pub store: RefCell<Store>,
    pub program: String,
    pub at: i64,
    pub limit: u64,
    pub tools: Option<Tools>,
    pub job: &'a Job,
}

impl Saver for Bundled<'_> {
    fn save(&self, material: &Material, how: Strategy) -> Result<u64, String> {
        self.job.starting(&material.url);
        let taken = match how {
            Strategy::Archive => self.archive(&material.url),
            Strategy::Mirror => self.mirror(&material.url),
            Strategy::Clone => self.cloned(&material.url),
            Strategy::Video => self.video(&material.url),
            Strategy::Direct => self.direct(&material.url),
        };
        self.job.stepped();
        taken
    }
}

impl Bundled<'_> {
    fn archive(&self, url: &str) -> Result<u64, String> {
        let page =
            page::save(url, &web(), renderer::current(), &Fetching::default()).map_err(say)?;
        self.put(url, "archive", &page.html)
    }

    fn direct(&self, url: &str) -> Result<u64, String> {
        let bytes = web().fetch(url).map_err(say)?;
        self.put(url, "file", &bytes)
    }

    fn mirror(&self, url: &str) -> Result<u64, String> {
        let mirrored = mirror::mirror(url, &web(), &Limits::default()).map_err(say)?;
        let corner = self.corner(url)?;
        for page in &mirrored.pages {
            std::fs::write(corner.join(&page.name), &page.html).map_err(say)?;
        }
        let entry = mirrored
            .pages
            .iter()
            .find(|page| page.url == url)
            .or_else(|| mirrored.pages.first())
            .ok_or_else(|| "зеркало вышло пустым".to_owned())?;
        std::fs::write(corner.join("index.html"), &entry.html).map_err(say)?;
        self.keep(url, "mirror")
    }

    fn cloned(&self, url: &str) -> Result<u64, String> {
        let corner = self.corner(url)?;
        std::fs::remove_dir_all(&corner).map_err(say)?;
        repo::clone(url, &corner, self.limit).map_err(say)?;
        self.keep(url, "repo")
    }

    fn video(&self, url: &str) -> Result<u64, String> {
        let tools = self
            .tools
            .as_ref()
            .ok_or_else(|| "нет yt-dlp или ffmpeg".to_owned())?;
        let into = self.corner(url)?.join(VIDEO);
        let mut watch = |_share: f32| {};
        video::download(
            tools,
            url,
            &into,
            Wanted::default(),
            &mut watch,
            &self.job.stop,
        )
        .map_err(say)?;
        self.keep(url, "video")
    }

    fn corner(&self, url: &str) -> Result<PathBuf, String> {
        self.store.borrow().corner(url).map_err(say)
    }

    fn put(&self, url: &str, kind: &str, bytes: &[u8]) -> Result<u64, String> {
        let fetched = Fetched {
            kind,
            bytes,
            etag: None,
            last_modified: None,
        };
        let stored = self
            .store
            .borrow_mut()
            .put(url, &self.program, &fetched, self.at)
            .map_err(say)?;
        Ok(stored.size)
    }

    fn keep(&self, url: &str, kind: &str) -> Result<u64, String> {
        let kept = self
            .store
            .borrow_mut()
            .keep(url, &self.program, kind, self.at)
            .map_err(say)?;
        Ok(kept.size)
    }
}

fn web() -> Web {
    Web { timeout: TIMEOUT }
}

fn say(error: impl Display) -> String {
    error.to_string()
}
