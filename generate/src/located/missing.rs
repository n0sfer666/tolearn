#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Missing {
    Node(String),
    Stage(String),
    Ungenerated(String),
}
