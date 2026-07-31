use tolearn_core::topic::Material;
use tolearn_offline::fresh::{Freshness, Mark, WINDOW, freshness};
use tolearn_offline::queue::{Step, plan};
use tolearn_offline::store::Held;

pub fn state(materials: &[Material], held: &[Option<Held>], now: i64) -> Freshness {
    let marks: Vec<Mark> = materials
        .iter()
        .zip(held)
        .filter_map(|(source, held)| mark(source, held.as_ref()))
        .collect();
    freshness(&marks, now, WINDOW)
}

fn mark(source: &Material, held: Option<&Held>) -> Option<Mark> {
    let Step::Take(how) = plan(source) else {
        return None;
    };
    Some(Mark {
        saved: held.is_some(),
        checkable: how.checkable(),
        checked_at: held.and_then(|held| held.checked_at),
    })
}
