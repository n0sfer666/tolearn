use crate::block::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub id: String,
    pub title: String,
    pub blocks: Vec<Block>,
    pub practice: Practice,
    pub questions: Vec<Question>,
}

impl Stage {
    pub fn every_block(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter().chain(&self.practice.task)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Practice {
    pub task: Vec<Block>,
    pub deliverable: String,
    pub constraints: Vec<Check>,
    pub acceptance: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub id: String,
    pub claim: String,
    pub check: Option<String>,
    pub expect: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub id: String,
    pub text: String,
    pub answer: String,
}
