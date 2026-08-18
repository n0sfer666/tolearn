use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub id: String,
    pub title: String,
    pub path: PathBuf,
    pub opened_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub program: Program,
    pub reachable: bool,
}
