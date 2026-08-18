use tolearn_core::topic::Material;

use super::strategy::Strategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Fetch,
    Same,
    Unchecked,
}

pub trait Checker {
    fn look(&self, material: &Material, how: Strategy) -> Result<Look, String>;
}

#[derive(Debug, Clone, Copy)]
pub struct Always;

impl Checker for Always {
    fn look(&self, _material: &Material, _how: Strategy) -> Result<Look, String> {
        Ok(Look::Fetch)
    }
}
