use tolearn_core::stage::Stage;

use super::flaw::Flaw;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Part {
    Theory,
    Practice,
    Questions,
    Block(String),
}

pub(super) fn parts(flaws: &[Flaw], stage: &Stage) -> Vec<Part> {
    let wanted: Vec<Part> = flaws.iter().filter_map(Flaw::part).collect();
    let within = |part: Part, blocks: &[tolearn_core::block::Block], id: &str| {
        wanted.contains(&part) && blocks.iter().any(|block| block.id == id)
    };
    let mut parts = Vec::new();
    for part in &wanted {
        let covered = match part {
            Part::Block(id) => {
                within(Part::Theory, &stage.blocks, id)
                    || within(Part::Practice, &stage.practice.task, id)
            }
            _ => false,
        };
        if !covered && !parts.contains(part) {
            parts.push(part.clone());
        }
    }
    parts
}
