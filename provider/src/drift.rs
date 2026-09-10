use std::collections::HashMap;

use sha2::{Digest, Sha256};

use crate::preset::preset;
use crate::types::Harness;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    pub removed: Vec<String>,
    pub added: Vec<String>,
    pub fingerprint: String,
}

pub fn fingerprint(advised: &[String]) -> String {
    let mut digest = Sha256::new();
    for arg in advised {
        digest.update(arg.as_bytes());
        digest.update([0]);
    }
    format!("{:x}", digest.finalize())
}

pub fn drift(harness: &Harness) -> Option<Drift> {
    let advised = preset(&harness.id)?.advised();
    if advised.is_empty() {
        return None;
    }
    let removed = missing(&harness.args, &advised);
    let added = missing(&advised, &harness.args);
    if removed.is_empty() && added.is_empty() {
        return None;
    }
    let fingerprint = fingerprint(&advised);
    if harness.dismissed_advice.as_deref() == Some(fingerprint.as_str()) {
        return None;
    }
    Some(Drift {
        removed,
        added,
        fingerprint,
    })
}

fn missing(from: &[String], baseline: &[String]) -> Vec<String> {
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for item in baseline {
        *counts.entry(item.as_str()).or_insert(0) += 1;
    }
    let mut leftover = Vec::new();
    for item in from {
        let count = counts.entry(item.as_str()).or_insert(0);
        if *count > 0 {
            *count -= 1;
        } else {
            leftover.push(item.clone());
        }
    }
    leftover
}
