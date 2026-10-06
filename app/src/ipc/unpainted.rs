use tolearn_generate::diagram::Painter;

#[derive(Debug, Clone, Copy)]
pub struct Unpainted;

impl Painter for Unpainted {
    fn paint(&self, _mermaid: &str) -> Result<String, String> {
        Err("окна для схем нет: схема остаётся кодом".to_owned())
    }
}
