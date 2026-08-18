use tolearn_core::topic::{Material, MaterialType};

use super::report::Skip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Mirror,
    Archive,
    Clone,
    Video,
    Direct,
}

impl Strategy {
    pub fn checkable(self) -> bool {
        matches!(self, Self::Mirror | Self::Archive | Self::Direct)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Take(Strategy),
    Skip(Skip),
}

pub fn plan(material: &Material) -> Step {
    match super::closed(material.liveness) {
        Some(why) => Step::Skip(why),
        None => Step::Take(strategy(material.kind)),
    }
}

fn strategy(kind: MaterialType) -> Strategy {
    match kind {
        MaterialType::Docs => Strategy::Mirror,
        MaterialType::Article | MaterialType::Post | MaterialType::Guide => Strategy::Archive,
        MaterialType::Spec | MaterialType::Rfc => Strategy::Archive,
        MaterialType::Repo => Strategy::Clone,
        MaterialType::Video => Strategy::Video,
        MaterialType::Paper => Strategy::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn каждый_тип_материала_знает_свой_способ() {
        assert_eq!(strategy(MaterialType::Docs), Strategy::Mirror);
        assert_eq!(strategy(MaterialType::Article), Strategy::Archive);
        assert_eq!(strategy(MaterialType::Rfc), Strategy::Archive);
        assert_eq!(strategy(MaterialType::Repo), Strategy::Clone);
        assert_eq!(strategy(MaterialType::Video), Strategy::Video);
        assert_eq!(strategy(MaterialType::Paper), Strategy::Direct);
    }
}
