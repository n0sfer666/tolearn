use tolearn_core::state::Lapse;

pub const UNPASSED_CHARS: usize = 1_500;

const HEAD: &str = "Незачтённые вопросы — на зачёте ученик ответил на них не полностью:";

pub fn block(lapses: &[Lapse]) -> Option<String> {
    let room = UNPASSED_CHARS.saturating_sub(HEAD.chars().count() + 1);
    let mut kept: Vec<String> = Vec::new();
    let mut used = 0;
    for lapse in lapses.iter().rev() {
        let entry = entry(lapse);
        let length = entry.chars().count() + usize::from(!kept.is_empty());
        if used + length > room {
            if kept.is_empty() {
                kept.push(entry.chars().take(room).collect());
            }
            break;
        }
        used += length;
        kept.push(entry);
    }
    if kept.is_empty() {
        return None;
    }
    kept.reverse();
    Some(format!("{HEAD}\n{}", kept.join("\n")))
}

fn entry(lapse: &Lapse) -> String {
    let asked = format!("- «{}»: {}", lapse.stage, lapse.question);
    if lapse.missed.is_empty() {
        return asked;
    }
    format!("{asked}\n  Упущено: {}", lapse.missed.join("; "))
}
