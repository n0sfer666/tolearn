use crate::sources::{PAGE_CHARS, excerpt};

use super::gathered::Gathered;

pub(super) fn cap(gathered: &Gathered, overflow: usize) -> usize {
    let lengths: Vec<usize> = gathered
        .pages
        .iter()
        .map(|visited| excerpt(&visited.page.text).chars().count())
        .collect();
    let within =
        |chars: usize| -> usize { lengths.iter().map(|length| (*length).min(chars)).sum() };
    let limit = within(PAGE_CHARS).saturating_sub(overflow);
    let (mut low, mut high) = (0, PAGE_CHARS);
    while low < high {
        let middle = (low + high).div_ceil(2);
        if within(middle) <= limit {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}
