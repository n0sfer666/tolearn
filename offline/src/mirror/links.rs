use std::collections::HashMap;

use markup5ever_rcdom::Handle;
use monolith::core::Options;
use monolith::html::{
    get_node_attr, get_node_name, html_to_dom, serialize_document, set_node_attr,
};
use url::Url;

use super::naming;

pub(super) fn hrefs(html: &str, base: &Url) -> Vec<Url> {
    let dom = html_to_dom(&html.as_bytes().to_vec(), "utf-8".to_string());
    let mut found = Vec::new();
    walk(&dom.document, &mut |node| {
        if get_node_name(node) != Some("a") {
            return;
        }
        if let Some(target) = get_node_attr(node, "href").and_then(|href| resolve(base, &href)) {
            found.push(target);
        }
    });
    found
}

pub(super) fn localize(html: &str, base: &Url, saved: &HashMap<String, String>) -> String {
    let dom = html_to_dom(&html.as_bytes().to_vec(), "utf-8".to_string());
    walk(&dom.document, &mut |node| {
        if get_node_name(node) != Some("a") {
            return;
        }
        let Some(href) = get_node_attr(node, "href") else {
            return;
        };
        let Some(target) = resolve(base, &href) else {
            return;
        };
        if let Some(name) = saved.get(target.as_str()) {
            set_node_attr(node, "href", Some(name.clone()));
        }
    });
    String::from_utf8_lossy(&serialize_document(
        dom,
        "utf-8".to_string(),
        &Options::default(),
    ))
    .to_string()
}

pub(super) fn local_name(url: &Url) -> String {
    let path = url.path().trim_matches('/');
    if path.is_empty() {
        return "index.html".to_string();
    }
    format!(
        "{}.html",
        naming::fitted(&naming::slugged(path, "page"), url)
    )
}

fn resolve(base: &Url, href: &str) -> Option<Url> {
    let href = href.trim();
    if href.is_empty() || href.starts_with('#') {
        return None;
    }
    let mut target = base.join(href).ok()?;
    if target.scheme() != "http" && target.scheme() != "https" {
        return None;
    }
    target.set_fragment(None);
    Some(target)
}

fn walk(node: &Handle, visit: &mut impl FnMut(&Handle)) {
    visit(node);
    for child in node.children.borrow().iter() {
        walk(child, visit);
    }
}
