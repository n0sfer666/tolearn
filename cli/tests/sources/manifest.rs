use std::collections::{BTreeMap, BTreeSet};

use super::parse;

pub fn manifest(dir: &str) -> toml::Table {
    parse(&format!("{dir}/Cargo.toml"))
}

pub fn dependencies(manifest: &toml::Table) -> BTreeSet<String> {
    let renames = workspace_renames();
    let mut names = BTreeSet::new();
    collect(manifest, &renames, &mut names);
    names
}

fn collect(table: &toml::Table, renames: &Renames, into: &mut BTreeSet<String>) {
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(entries) = table.get(section).and_then(toml::Value::as_table) else {
            continue;
        };
        for (key, entry) in entries {
            into.insert(package_of(key, entry, renames));
        }
    }
    let Some(targets) = table.get("target").and_then(toml::Value::as_table) else {
        return;
    };
    for platform in targets.values().filter_map(toml::Value::as_table) {
        collect(platform, renames, into);
    }
}

fn package_of(key: &str, entry: &toml::Value, renames: &Renames) -> String {
    let entry = entry.as_table();
    let renamed = entry
        .and_then(|entry| entry.get("package"))
        .and_then(toml::Value::as_str);
    if let Some(renamed) = renamed {
        return renamed.to_owned();
    }
    let inherited = entry
        .and_then(|entry| entry.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    if inherited && let Some(renamed) = renames.get(key) {
        return renamed.clone();
    }
    key.to_owned()
}

type Renames = BTreeMap<String, String>;

fn workspace_renames() -> Renames {
    let root = parse("Cargo.toml");
    let entries = root
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table);
    let Some(entries) = entries else {
        return Renames::new();
    };
    entries
        .iter()
        .filter_map(|(key, entry)| {
            let renamed = entry.as_table()?.get("package")?.as_str()?;
            Some((key.clone(), renamed.to_owned()))
        })
        .collect()
}
