pub(crate) fn latest(entries: Vec<String>, room: usize, gap: usize) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    let mut used = 0;
    for entry in entries.into_iter().rev() {
        let length = entry.chars().count() + if kept.is_empty() { 0 } else { gap };
        if used + length > room {
            if kept.is_empty() {
                kept.push(entry.chars().take(room).collect());
            }
            break;
        }
        used += length;
        kept.push(entry);
    }
    kept.reverse();
    kept
}
