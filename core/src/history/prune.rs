use super::types::Kept;

pub fn prune(kept: &[Kept], depth: u32, budget: u64) -> Vec<u32> {
    let mut order: Vec<&Kept> = kept.iter().collect();
    order.sort_by_key(|version| version.n);

    if depth == 0 {
        return order.iter().map(|version| version.n).collect();
    }

    let over = order.len().saturating_sub(depth as usize);
    let mut dropped: Vec<u32> = order.iter().take(over).map(|version| version.n).collect();

    let mut left: u64 = order.iter().skip(over).map(|version| version.bytes).sum();
    for version in order.iter().skip(over) {
        if left <= budget || version.n == order[order.len() - 1].n {
            break;
        }
        left = left.saturating_sub(version.bytes);
        dropped.push(version.n);
    }

    dropped
}
