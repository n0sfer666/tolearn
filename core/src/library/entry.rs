use super::refusal::Refusal;
use crate::program::Tree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub directory: String,
    pub program: Result<Tree, Refusal>,
}
