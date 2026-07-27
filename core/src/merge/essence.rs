use super::report::Part;
use crate::topic::Topic;

pub(super) fn changed(before: &Topic, after: &Topic) -> Vec<Part> {
    let mut parts = Vec::new();
    if before.depends_on != after.depends_on {
        parts.push(Part::DependsOn);
    }
    if before.outcomes != after.outcomes {
        parts.push(Part::Outcomes);
    }
    if before.misconceptions != after.misconceptions {
        parts.push(Part::Misconceptions);
    }
    if before.practice != after.practice {
        parts.push(Part::Practice);
    }
    if before.questions != after.questions {
        parts.push(Part::Questions);
    }
    if before.exam != after.exam {
        parts.push(Part::Exam);
    }
    parts
}
