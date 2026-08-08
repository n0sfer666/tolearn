const PENDING: &str = "generated: false";
const DONE: &str = "generated: true";
const STAMP: &str = "generated_at:";

pub fn stamped(roadmap: &str, today: &str) -> String {
    let mut fixed = String::with_capacity(roadmap.len());
    for line in roadmap.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        if body.starts_with(STAMP) {
            fixed.push_str(STAMP);
            fixed.push(' ');
            fixed.push_str(today);
            fixed.push_str(&line[body.len()..]);
        } else {
            fixed.push_str(line);
        }
    }
    fixed
}

pub fn generated(roadmap: &str) -> String {
    let mut marked = String::with_capacity(roadmap.len());
    for line in roadmap.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        if body.trim_start().trim_start_matches("- ").trim() == PENDING {
            let indent = body.len() - body.trim_start().len();
            marked.push_str(&body[..indent]);
            if body.trim_start().starts_with("- ") {
                marked.push_str("- ");
            }
            marked.push_str(DONE);
            marked.push_str(&line[body.len()..]);
        } else {
            marked.push_str(line);
        }
    }
    marked
}
