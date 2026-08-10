use std::path::Path;

use super::piece::Piece;
use super::seen::seen;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cost {
    pub materials: usize,
    pub held: usize,
    pub used: u64,
    pub budget: u64,
    pub spare: u64,
    pub need: u64,
    pub tight: bool,
}

pub fn cost(root: &Path, budget: u64, pieces: &[Piece]) -> Cost {
    let seen = seen(root, budget);
    let held: Vec<u64> = pieces
        .iter()
        .filter_map(|piece| seen.held(&piece.material.url))
        .map(|held| held.size)
        .collect();
    let need = weighed(pieces.len() - held.len(), &held);
    let used = seen.size();
    Cost {
        materials: pieces.len(),
        held: held.len(),
        used,
        budget,
        spare: seen.spare(),
        need,
        tight: need > budget.saturating_sub(used),
    }
}

fn weighed(missing: usize, held: &[u64]) -> u64 {
    let count = u64::try_from(held.len()).unwrap_or(u64::MAX);
    if count == 0 {
        return 0;
    }
    let average = held.iter().sum::<u64>() / count;
    average.saturating_mul(u64::try_from(missing).unwrap_or(u64::MAX))
}
