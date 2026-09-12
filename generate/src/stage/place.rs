use tolearn_core::program::{Program, StageRow};

#[derive(Debug, Clone, Copy)]
pub struct Place<'a> {
    pub program: &'a Program,
    pub row: &'a StageRow,
    pub index: usize,
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
            })
    }

    pub fn first(&self) -> bool {
        self.index == 0
    }
}
