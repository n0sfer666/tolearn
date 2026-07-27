use std::path::Path;

use serde_json::{Value, json};
use tolearn_core::scan::Scan;

use crate::bundle;
use crate::error::CliError;
use crate::out::Output;

pub fn run(root: &Path) -> Result<Output, CliError> {
    let scan = bundle::read(root)?;
    Ok(Output::new(text(&scan), body(&scan)))
}

fn text(scan: &Scan) -> String {
    let mut out = format!(
        "{} — тем разобрано: {}, не сгенерировано: {}, битых: {}\n",
        scan.roadmap.title,
        scan.topics.len(),
        scan.absent.len(),
        scan.broken.len()
    );
    for absent in &scan.absent {
        out.push_str(&format!("  нет файла: {} ({})\n", absent.id, absent.file));
    }
    for broken in &scan.broken {
        out.push_str(&format!(
            "  не читается: {} — {}\n",
            broken.id, broken.error
        ));
    }
    out
}

fn body(scan: &Scan) -> Value {
    json!({
        "root": scan.root.display().to_string(),
        "roadmap": scan.roadmap.id,
        "topics": scan.topics.iter().map(|topic| json!({
            "id": topic.id,
            "title": topic.title,
            "stage": topic.stage,
        })).collect::<Vec<Value>>(),
        "absent": scan.absent.iter().map(|absent| json!({
            "id": absent.id,
            "file": absent.file,
            "stage": absent.stage,
            "generated": absent.generated,
        })).collect::<Vec<Value>>(),
        "broken": scan.broken.iter().map(|broken| json!({
            "id": broken.id,
            "file": broken.file,
            "error": broken.error.to_string(),
        })).collect::<Vec<Value>>(),
    })
}
