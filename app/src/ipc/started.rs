use super::dto::dto;
use super::planned::PlanView;
use super::shape::Shape;

pub const STEP_EVENT: &str = "generation-step";

dto!(StartProgramIn {
    request: String,
    level: String,
    plan: PlanView,
});
dto!(StartProgramOut {
    program: String,
    node: String,
    stage: String,
});
dto!(CancelGenerationIn {});
dto!(CancelGenerationOut { cancelled: bool });
dto!(GenerationStep {
    step: String,
    state: String,
    round: u32,
    of: u32,
});

pub fn shapes() -> Vec<Shape> {
    vec![
        StartProgramIn::shape(),
        StartProgramOut::shape(),
        CancelGenerationIn::shape(),
        CancelGenerationOut::shape(),
        GenerationStep::shape(),
    ]
}
