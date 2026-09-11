use super::types::Program;
use super::violation::Violation;
use crate::Hours;

pub fn check(program: &Program) -> Vec<Violation> {
    let map = &program.map;
    let mut found = Vec::new();
    if !map.stages.is_empty() && !map.children.is_empty() {
        found.push(Violation::NodeWithStages {
            program: program.uuid.clone(),
        });
    }
    let stages = map.stages.iter().map(|row| (&row.id, row.hours));
    let children = map.children.iter().map(|row| (&row.uuid, row.hours));
    for (row, Hours { min, max }) in stages.chain(children) {
        if min > max {
            found.push(Violation::HoursReversed {
                row: row.clone(),
                min,
                max,
            });
        }
    }
    found
}
