use super::variant::Variant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fork {
    pub variants: Vec<Variant>,
}
