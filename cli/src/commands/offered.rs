use std::path::Path;

use serde_json::{Value, json};
use tolearn_generate::fork::{After, Fork};

use crate::out::Output;
use crate::session::hours;

pub fn offered(data: &Path, after: &After<'_>, fork: &Fork) -> Output {
    let numbered = || {
        fork.variants
            .iter()
            .enumerate()
            .map(|(index, variant)| (index.saturating_add(1), variant))
    };
    let lines: Vec<String> = numbered()
        .map(|(choice, variant)| {
            let mut parts = vec![
                format!("{choice}. {} «{}»", variant.row.id, variant.row.title),
                hours(variant.row.hours),
            ];
            if variant.recommended {
                parts.push("рекомендую".to_owned());
            }
            format!("  {} — {}", parts.join(" · "), variant.why)
        })
        .collect();
    let text = format!(
        "развилка после этапа {}:\n{}\n\nвыбор: tolearn next {} --choice <номер>",
        after.stage,
        lines.join("\n"),
        data.display()
    );
    let variants: Vec<Value> = numbered()
        .map(|(choice, variant)| {
            json!({
                "choice": choice,
                "id": variant.row.id,
                "title": variant.row.title,
                "hours": { "min": variant.row.hours.min, "max": variant.row.hours.max },
                "why": variant.why,
                "recommended": variant.recommended,
            })
        })
        .collect();
    let json = json!({
        "program": after.program,
        "node": after.node,
        "after": after.stage,
        "variants": variants,
    });
    Output::new(text, json)
}
