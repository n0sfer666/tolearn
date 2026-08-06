const PENDING: &str = "generated: false";
const DONE: &str = "generated: true";

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
