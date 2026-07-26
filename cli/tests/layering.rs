#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "repository gate: a panic here is the report"
)]

mod repo;
mod sources;

use std::collections::BTreeSet;

use sources::document::documented_tree;
use sources::lockfile::{locked_graph, reachable_from};
use sources::manifest::{dependencies, manifest};

struct Layer {
    dir: &'static str,
    package: &'static str,
    allowed_internal: &'static [&'static str],
    forbidden_markers: &'static [&'static str],
}

const SHELL_MARKERS: &[&str] = &["tauri", "wry", "webkit", "objc"];
const NON_CRATE_DIRS: &[&str] = &["ui", "docs", "examples", "fixtures"];
const INHERITED_FIELDS: &[&str] = &[
    "version",
    "edition",
    "rust-version",
    "license",
    "repository",
    "publish",
];

const LAYERS: &[Layer] = &[
    Layer {
        dir: "core",
        package: "tolearn-core",
        allowed_internal: &[],
        forbidden_markers: SHELL_MARKERS,
    },
    Layer {
        dir: "runner",
        package: "tolearn-runner",
        allowed_internal: &["tolearn-core"],
        forbidden_markers: SHELL_MARKERS,
    },
    Layer {
        dir: "offline",
        package: "tolearn-offline",
        allowed_internal: &["tolearn-core"],
        forbidden_markers: SHELL_MARKERS,
    },
    Layer {
        dir: "cli",
        package: "tolearn-cli",
        allowed_internal: &["tolearn-core", "tolearn-runner", "tolearn-offline"],
        forbidden_markers: SHELL_MARKERS,
    },
    Layer {
        dir: "app",
        package: "tolearn-app",
        allowed_internal: &["tolearn-core", "tolearn-runner", "tolearn-offline"],
        forbidden_markers: &[],
    },
];

#[test]
fn workspace_members_match_documented_tree() {
    let documented: BTreeSet<String> = documented_tree()
        .into_iter()
        .filter(|dir| !NON_CRATE_DIRS.contains(&dir.as_str()))
        .collect();
    let declared: BTreeSet<String> = LAYERS.iter().map(|layer| layer.dir.to_owned()).collect();
    let members: BTreeSet<String> = manifest(".")["workspace"]["members"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(toml::Value::as_str)
        .map(str::to_owned)
        .collect();

    assert_eq!(documented, declared, "docs/architecture.md#структура");
    assert_eq!(members, declared, "Cargo.toml workspace members");
}

#[test]
fn every_crate_declares_its_documented_package_name() {
    for layer in LAYERS {
        let manifest = manifest(layer.dir);
        let name = manifest["package"]["name"].as_str();
        assert_eq!(name, Some(layer.package), "{}/Cargo.toml", layer.dir);
    }
}

#[test]
fn dependency_direction_is_downwards() {
    let internal: BTreeSet<&str> = LAYERS.iter().map(|layer| layer.package).collect();

    for layer in LAYERS {
        let allowed: BTreeSet<&str> = layer.allowed_internal.iter().copied().collect();
        for dependency in dependencies(&manifest(layer.dir)) {
            let dependency = dependency.as_str();
            assert!(
                !internal.contains(dependency) || allowed.contains(dependency),
                "{} depends on {dependency}, which is not below it",
                layer.package
            );
        }
    }
}

#[test]
fn lower_layers_do_not_know_about_the_shell() {
    for layer in LAYERS {
        for dependency in dependencies(&manifest(layer.dir)) {
            for marker in layer.forbidden_markers {
                assert!(
                    !dependency.contains(marker),
                    "{} depends on {dependency}: the shell lives in app only (ADR-003)",
                    layer.package
                );
            }
        }
    }
}

#[test]
fn the_shell_does_not_reach_lower_layers_transitively() {
    let graph = locked_graph();

    for layer in LAYERS {
        for dependency in reachable_from(layer.package, &graph) {
            for marker in layer.forbidden_markers {
                assert!(
                    !dependency.contains(marker),
                    "{} pulls in {dependency} transitively (ADR-003)",
                    layer.package
                );
            }
        }
    }
}

#[test]
fn every_crate_inherits_workspace_settings() {
    for layer in LAYERS {
        let manifest = manifest(layer.dir);
        let lints = manifest
            .get("lints")
            .and_then(|lints| lints.get("workspace"))
            .and_then(toml::Value::as_bool);
        assert_eq!(lints, Some(true), "{}/Cargo.toml [lints]", layer.dir);

        for field in INHERITED_FIELDS {
            let inherited = manifest["package"]
                .get(field)
                .and_then(|value| value.get("workspace"))
                .and_then(toml::Value::as_bool);
            assert_eq!(inherited, Some(true), "{}/Cargo.toml {field}", layer.dir);
        }
    }
}
