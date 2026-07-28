use crate::progress::Status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub layer: usize,
    pub depends_on: Vec<String>,
    pub blocked_by: Vec<String>,
    pub unlocks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub nodes: Vec<Node>,
}
