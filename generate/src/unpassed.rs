use tolearn_core::state::Lapse;

use crate::latest::latest;

pub const UNPASSED_CHARS: usize = 1_500;

const HEAD: &str = "Незачтённые вопросы — на зачёте ученик ответил на них не полностью:";

pub fn block(lapses: &[Lapse]) -> Option<String> {
    let room = UNPASSED_CHARS.saturating_sub(HEAD.chars().count() + 1);
    let kept = latest(lapses.iter().map(entry).collect(), room, 1);
    if kept.is_empty() {
        return None;
    }
    Some(format!("{HEAD}\n{}", kept.join("\n")))
}

fn entry(lapse: &Lapse) -> String {
    let asked = format!("- «{}»: {}", lapse.stage, lapse.question);
    if lapse.missed.is_empty() {
        return asked;
    }
    format!("{asked}\n  Упущено: {}", lapse.missed.join("; "))
}
