use crate::date::Date;

const PENDING: &str = "generated: false";
const DONE: &str = "generated: true";
const STAMP: &str = "generated_at:";

pub fn stamped(roadmap: &str, today: Date) -> String {
    rewritten(roadmap, |body| {
        body.starts_with(STAMP).then(|| format!("{STAMP} {today}"))
    })
}

pub fn generated(roadmap: &str) -> String {
    rewritten(roadmap, |body| {
        let bare = body.trim_start();
        if bare.trim_start_matches("- ").trim() != PENDING {
            return None;
        }
        let indent = &body[..body.len() - bare.len()];
        let dash = if bare.starts_with("- ") { "- " } else { "" };
        Some(format!("{indent}{dash}{DONE}"))
    })
}

fn rewritten(roadmap: &str, mend: impl Fn(&str) -> Option<String>) -> String {
    let mut fixed = String::with_capacity(roadmap.len());
    for line in roadmap.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        match mend(body) {
            Some(mended) => {
                fixed.push_str(&mended);
                fixed.push_str(&line[body.len()..]);
            }
            None => fixed.push_str(line),
        }
    }
    fixed
}
