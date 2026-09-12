use std::cell::Cell;

use tolearn_generate::diagram::Painter;

pub const UNPAINTED: &str = "окна для схем нет";

#[derive(Debug, Default)]
pub struct Unpainted(Cell<usize>);

impl Unpainted {
    pub fn count(&self) -> usize {
        self.0.get()
    }
}

impl Painter for Unpainted {
    fn paint(&self, _mermaid: &str) -> Result<String, String> {
        self.0.set(self.0.get().saturating_add(1));
        Err(format!("{UNPAINTED}: схема остаётся кодом"))
    }
}
