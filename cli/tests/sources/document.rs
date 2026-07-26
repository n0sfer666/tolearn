use std::collections::BTreeSet;

use super::read;

pub fn documented_tree() -> BTreeSet<String> {
    let text = read("docs/architecture.md");
    let section = text
        .split("## Структура")
        .nth(1)
        .unwrap_or_else(|| panic!("docs/architecture.md: no `## Структура` section"));
    let block = section
        .split("```")
        .nth(1)
        .unwrap_or_else(|| panic!("docs/architecture.md: no tree block"));
    block
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter_map(|entry| entry.strip_suffix('/'))
        .map(str::to_owned)
        .collect()
}
