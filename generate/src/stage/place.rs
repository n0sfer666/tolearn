use tolearn_core::program::{Program, StageRow};
use tolearn_core::state::Lapse;

#[derive(Debug, Clone, Copy)]
pub struct Place<'a> {
    pub program: &'a Program,
    pub row: &'a StageRow,
    pub index: usize,
    pub lapses: &'a [Lapse],
}

impl<'a> Place<'a> {
    pub fn find(program: &'a Program, stage: &str) -> Option<Self> {
        program
            .map
            .stages
            .iter()
            .enumerate()
            .find(|(_, row)| row.id == stage)
            .map(|(index, row)| Self {
                program,
                row,
                index,
                lapses: &[],
            })
    }

    pub fn first(&self) -> bool {
        self.index == 0
    }
}
