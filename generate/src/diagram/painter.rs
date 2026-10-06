pub trait Painter {
    fn paint(&self, mermaid: &str) -> Result<String, String>;
}
