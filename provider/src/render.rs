use crate::types::Provider;

pub const SCHEMA: &str = "tolearn/provider/v1";

pub fn text(provider: &Provider) -> String {
    format!(
        "schema: {SCHEMA}\n\
         enabled: {}\n\
         flavor: {}\n\
         endpoint: {}\n\
         model: {}\n",
        provider.enabled,
        provider.flavor.label(),
        quoted(&provider.endpoint),
        quoted(&provider.model),
    )
}

fn quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
