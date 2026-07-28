mod links;
mod robots;

pub use robots::Robots;

use std::collections::{HashMap, HashSet, VecDeque};

use url::Url;

use crate::page::{PageError, Source};

#[derive(Debug, Clone)]
pub struct Limits {
    pub depth: usize,
    pub pages: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: 2,
            pages: 50,
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
    let seeds = listed.unwrap_or_else(|| vec![start.clone()]);

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
        let html = String::from_utf8_lossy(&bytes).to_string();
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
    let pages = raw
        .iter()
        .map(|(url, html)| Mirrored {
            url: url.to_string(),
            name: links::local_name(url),
            html: links::localize(html, url, &names),
        })
        .collect();

    Ok(Mirror { pages, skipped })
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
