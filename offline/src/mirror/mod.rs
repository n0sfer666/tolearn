mod assets;
mod links;
mod markup;
mod robots;

pub use assets::{Asset, Weight};
pub use robots::Robots;

use std::collections::{HashMap, HashSet, VecDeque};

use url::Url;

use crate::page::{PageError, Source};

const EACH: u64 = 16 * 1024 * 1024;
const TOTAL: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Limits {
    pub depth: usize,
    pub pages: usize,
    pub weight: assets::Weight,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: 2,
            pages: 50,
            weight: assets::Weight {
                each: EACH,
                total: TOTAL,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Skip {
    Robots,
    Domain,
    Depth,
    Cap,
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct Mirrored {
    pub url: String,
    pub name: String,
    pub html: String,
}

#[derive(Debug, Clone)]
pub struct Mirror {
    pub pages: Vec<Mirrored>,
    pub assets: Vec<Asset>,
    pub skipped: Vec<(String, Skip)>,
}

pub fn mirror(root: &str, source: &dyn Source, limits: &Limits) -> Result<Mirror, PageError> {
    let start = Url::parse(root)
        .map_err(|error| PageError::Unreachable(root.to_string(), error.to_string()))?;
    let robots = text(source, &join(&start, "/robots.txt"))
        .map(|text| Robots::read(&text))
        .unwrap_or_default();
    let listed = listed(source, &start);
    let walking = listed.is_none();
    let mut seeds = listed.unwrap_or_default();
    seeds.insert(0, start.clone());

    let mut skipped = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut raw: Vec<(Url, String)> = Vec::new();
    let mut queue: VecDeque<(Url, usize)> = seeds.into_iter().map(|url| (url, 0)).collect();

    while let Some((url, depth)) = queue.pop_front() {
        if !seen.insert(url.as_str().to_string()) {
            continue;
        }
        if url.host_str() != start.host_str() {
            skipped.push((url.to_string(), Skip::Domain));
            continue;
        }
        if !robots.allows(url.path()) {
            skipped.push((url.to_string(), Skip::Robots));
            continue;
        }
        if raw.len() >= limits.pages {
            skipped.push((url.to_string(), Skip::Cap));
            continue;
        }
        let Ok(bytes) = source.fetch(url.as_str()) else {
            if url == start {
                return Err(PageError::Unreachable(
                    url.to_string(),
                    "корень зеркала не открылся".to_string(),
                ));
            }
            skipped.push((url.to_string(), Skip::Unreachable));
            continue;
        };
        let raw_text = String::from_utf8_lossy(&bytes).to_string();
        let html = markup::as_html(&raw_text, &titled(&url)).unwrap_or(raw_text);
        if walking {
            for target in links::hrefs(&html, &url) {
                if depth < limits.depth {
                    queue.push_back((target, depth + 1));
                } else if !seen.contains(target.as_str()) {
                    skipped.push((target.to_string(), Skip::Depth));
                }
            }
        }
        raw.push((url, html));
    }

    let names: HashMap<String, String> = raw
        .iter()
        .map(|(url, _)| (url.as_str().to_string(), links::local_name(url)))
        .collect();
    let (assets, local) = carried(source, &raw, &limits.weight);
    let pages = raw
        .iter()
        .map(|(url, html)| Mirrored {
            url: url.to_string(),
            name: links::local_name(url),
            html: assets::relink(&links::localize(html, url, &names), url, &local),
        })
        .collect();

    Ok(Mirror {
        pages,
        assets,
        skipped,
    })
}

fn carried(
    source: &dyn Source,
    raw: &[(Url, String)],
    weight: &Weight,
) -> (Vec<Asset>, HashMap<String, String>) {
    let mut seen = HashSet::new();
    let wanted: Vec<Url> = raw
        .iter()
        .flat_map(|(url, html)| assets::wanted(html, url))
        .filter(|url| seen.insert(url.as_str().to_string()))
        .collect();
    let taken = assets::fetch(source, &wanted, weight);
    let local = taken
        .iter()
        .map(|(url, asset)| (url.clone(), asset.name.clone()))
        .collect();
    let assets = taken.into_iter().map(|(_, asset)| asset).collect();
    (assets, local)
}

fn titled(url: &Url) -> String {
    url.path_segments()
        .and_then(|mut parts| parts.next_back())
        .filter(|last| !last.is_empty())
        .unwrap_or_else(|| url.as_str())
        .to_string()
}

fn listed(source: &dyn Source, start: &Url) -> Option<Vec<Url>> {
    let manifest = text(source, &join(start, "/llms.txt"))?;
    let found: Vec<Url> = manifest
        .lines()
        .filter_map(|line| line.split_once("](").map(|(_, tail)| tail))
        .filter_map(|tail| tail.split(')').next())
        .filter_map(|href| start.join(href).ok())
        .collect();
    if found.is_empty() { None } else { Some(found) }
}

fn text(source: &dyn Source, url: &Url) -> Option<String> {
    let bytes = source.fetch(url.as_str()).ok()?;
    Some(String::from_utf8_lossy(&bytes).to_string())
}

fn join(start: &Url, path: &str) -> Url {
    start.join(path).unwrap_or_else(|_| start.clone())
}
