use crate::types::{Harness, Http, Provider};

pub const SCHEMA: &str = "tolearn/provider/v2";

pub fn text(provider: &Provider) -> String {
    format!(
        "schema: {SCHEMA}\n\
         enabled: {}\n\
         active: {}\n\
         local:\n{}\
         remote:\n{}\
         harness:\n{}",
        provider.enabled,
        provider.active.label(),
        http(&provider.local),
        http(&provider.remote),
        harness(&provider.harness),
    )
}

fn http(http: &Http) -> String {
    format!(
        "  endpoint: {}\n  api: {}\n  model: {}\n",
        quoted(&http.endpoint),
        http.api.label(),
        quoted(&http.model),
    )
}

fn harness(harness: &Harness) -> String {
    format!(
        "  id: {}\n  command: {}\n  args:{}\n  timeout_secs: {}\n",
        quoted(&harness.id),
        quoted(&harness.command),
        args(&harness.args),
        harness.timeout_secs,
    )
}

fn args(args: &[String]) -> String {
    if args.is_empty() {
        return " []".to_owned();
    }
    args.iter()
        .map(|arg| format!("\n    - {}", quoted(arg)))
        .collect()
}

fn quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
