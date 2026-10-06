use std::collections::{BTreeMap, BTreeSet};

use super::types::Program;
use crate::stage::Stage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub program: Program,
    pub stages: BTreeMap<String, Stage>,
    pub assets: BTreeSet<String>,
    pub children: BTreeMap<String, Tree>,
}

impl Tree {
    pub fn files(&self) -> Vec<String> {
        let mut files = vec!["program.yaml".to_owned()];
        files.extend(self.stages.keys().map(|id| format!("stages/{id}.yaml")));
        files.extend(self.assets.iter().cloned());
        for (uuid, child) in &self.children {
            files.extend(
                child
                    .files()
                    .into_iter()
                    .map(|file| format!("children/{uuid}/{file}")),
            );
        }
        files
    }

    pub fn uuids(&self) -> BTreeSet<String> {
        let mut found: BTreeSet<String> = self.children.values().flat_map(Tree::uuids).collect();
        found.insert(self.program.uuid.clone());
        found.extend(self.program.map.children.iter().map(|row| row.uuid.clone()));
        found
    }

    pub fn every_stage(&self) -> Vec<(&str, &str)> {
        let node = self.program.uuid.as_str();
        let mut found: Vec<(&str, &str)> = self
            .program
            .map
            .stages
            .iter()
            .filter(|row| self.stages.contains_key(&row.id))
            .map(|row| (node, row.id.as_str()))
            .collect();
        for row in &self.program.map.children {
            if let Some(child) = self.children.get(&row.uuid) {
                found.extend(child.every_stage());
            }
        }
        found
    }
}
