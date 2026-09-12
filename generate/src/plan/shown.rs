use serde_json::json;

use super::types::Plan;

pub(super) fn shown(plan: &Plan) -> String {
    let stages: Vec<_> = plan
        .stages
        .iter()
        .map(|row| json!({ "id": row.id, "title": row.title, "hours": [row.hours.min, row.hours.max] }))
        .collect();
    let children: Vec<_> = plan
        .children
        .iter()
        .map(|row| json!({ "title": row.title, "goal": row.goal, "hours": [row.hours.min, row.hours.max] }))
        .collect();
    json!({
        "title": plan.title,
        "slug": plan.slug,
        "goal": plan.goal,
        "volatility": plan.volatility.label(),
        "stages": stages,
        "children": children,
    })
    .to_string()
}
