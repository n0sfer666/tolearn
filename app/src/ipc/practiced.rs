use super::dto::dto;
use super::shape::Shape;

dto!(TickIn {
    program: String,
    node: String,
    stage: String,
    claim: String,
    on: bool,
});
dto!(TickOut { ticks: Vec<String> });
dto!(WorkdirIn {
    program: String,
    path: String,
});
dto!(WorkdirOut { workdir: String });
dto!(CheckClaimIn {
    program: String,
    node: String,
    stage: String,
    claim: String,
});
dto!(CheckClaimOut {
    outcome: String,
    code: Option<i32>,
    stdout: String,
    stderr: String,
    truncated: bool,
});

pub fn shapes() -> Vec<Shape> {
    vec![
        TickIn::shape(),
        TickOut::shape(),
        WorkdirIn::shape(),
        WorkdirOut::shape(),
        CheckClaimIn::shape(),
        CheckClaimOut::shape(),
    ]
}
