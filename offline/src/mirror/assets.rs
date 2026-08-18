use std::collections::HashMap;

use markup5ever_rcdom::Handle;
use monolith::core::Options;
use monolith::html::{
    get_node_attr, get_node_name, html_to_dom, serialize_document, set_node_attr,
};
use url::Url;

use super::naming;
use crate::page::Source;

const CARRYING: [(&str, &str); 6] = [
    ("img", "src"),
    ("img", "data-src"),
    ("video", "src"),
    ("video", "poster"),
    ("audio", "src"),
    ("source", "src"),
];

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Weight {
    pub each: u64,
    pub total: u64,
}

pub(super) fn wanted(html: &str, base: &Url) -> Vec<Url> {
    let dom = html_to_dom(&html.as_bytes().to_vec(), "utf-8".to_string());
    let mut found = Vec::new();
    walk(&dom.document, &mut |node| {
        for target in addresses(node, base) {
            found.push(target);
        }
    });
    found
}

pub(super) fn relink(html: &str, base: &Url, saved: &HashMap<String, String>) -> String {
    let dom = html_to_dom(&html.as_bytes().to_vec(), "utf-8".to_string());
    walk(&dom.document, &mut |node| {
        let Some(name) = get_node_name(node) else {
            return;
        };
        for (tag, attribute) in CARRYING {
            if tag != name {
                continue;
            }
            rewrite(node, attribute, base, saved);
        }
    });
    String::from_utf8_lossy(&serialize_document(
        dom,
        "utf-8".to_string(),
        &Options::default(),
    ))
    .to_string()
}

pub(super) fn fetch(source: &dyn Source, urls: &[Url], weight: &Weight) -> Vec<(String, Asset)> {
    let mut taken = 0u64;
    let mut got = Vec::new();
    for url in urls {
        let Ok(bytes) = source.fetch(url.as_str()) else {
            continue;
        };
        let size = bytes.len() as u64;
        if size > weight.each || taken + size > weight.total {
            continue;
        }
        taken += size;
        got.push((
            url.as_str().to_string(),
            Asset {
                name: name(url),
                bytes,
            },
        ));
    }
    got
}

fn addresses(node: &Handle, base: &Url) -> Vec<Url> {
    let Some(name) = get_node_name(node) else {
        return Vec::new();
    };
    CARRYING
        .iter()
        .filter(|(tag, _)| *tag == name)
        .filter_map(|(_, attribute)| get_node_attr(node, attribute))
        .filter_map(|value| resolve(base, &value))
        .collect()
}

fn rewrite(node: &Handle, attribute: &str, base: &Url, saved: &HashMap<String, String>) {
    let Some(value) = get_node_attr(node, attribute) else {
        return;
    };
    let Some(target) = resolve(base, &value) else {
        return;
    };
    if let Some(name) = saved.get(target.as_str()) {
        set_node_attr(node, attribute, Some(name.clone()));
    }
}

fn resolve(base: &Url, value: &str) -> Option<Url> {
    let value = value.trim();
    if value.is_empty() || value.starts_with("data:") {
        return None;
    }
    let mut target = base.join(value).ok()?;
    if target.scheme() != "http" && target.scheme() != "https" {
        return None;
    }
    target.set_fragment(None);
    Some(target)
}

fn name(url: &Url) -> String {
    let path = url.path().trim_matches('/');
    let (stem, kind) = match path.rsplit_once('.') {
        Some((stem, kind)) if kind.chars().all(char::is_alphanumeric) => (stem, Some(kind)),
        _ => (path, None),
    };
    let raw = format!("{stem}-{}", url.query().unwrap_or_default());
    let slug = naming::fitted(&naming::slugged(&raw, "asset"), url);
    match kind {
        Some(kind) => format!("{slug}.{kind}"),
        None => slug,
    }
}

fn walk(node: &Handle, visit: &mut impl FnMut(&Handle)) {
    visit(node);
    for child in node.children.borrow().iter() {
        walk(child, visit);
    }
}
