use super::dto::dto;
use super::shape::Shape;

dto!(RegenerateStageIn {
    program: String,
    node: String,
    stage: String,
});
dto!(RegenerateStageOut { stage: String });

pub fn shapes() -> Vec<Shape> {
    vec![RegenerateStageIn::shape(), RegenerateStageOut::shape()]
}
