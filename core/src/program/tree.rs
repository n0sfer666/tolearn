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
