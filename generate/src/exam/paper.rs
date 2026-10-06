use std::collections::BTreeMap;

use tolearn_core::stage::Stage;

#[derive(Debug, Clone, Copy)]
pub struct Paper<'a> {
    pub program: &'a str,
    pub node: &'a str,
    pub stage: &'a Stage,
    pub level: &'a str,
    pub locale: &'a str,
    pub answers: &'a BTreeMap<String, String>,
}
