const FENCE: &str = "```";

pub fn pick(answer: &str, schema: &str) -> Option<String> {
    let mut blocks = fenced(answer);
    if blocks.is_empty() {
        blocks.push(answer.to_owned());
    }
    blocks.into_iter().find(|block| declares(block, schema))
}

pub fn fenced(answer: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut inside: Option<Vec<&str>> = None;
    for line in answer.lines() {
        match (line.trim_start().starts_with(FENCE), inside.as_mut()) {
            (true, None) => inside = Some(Vec::new()),
            (true, Some(_)) => closed(&mut inside, &mut blocks),
            (false, Some(body)) => body.push(line),
            (false, None) => {}
        }
    }
    closed(&mut inside, &mut blocks);
    blocks
}

fn closed(inside: &mut Option<Vec<&str>>, blocks: &mut Vec<String>) {
    if let Some(body) = inside.take() {
        blocks.push(format!("{}\n", body.join("\n")));
    }
}

fn declares(block: &str, schema: &str) -> bool {
    block.lines().any(|line| {
        line.strip_prefix("schema:")
            .is_some_and(|rest| rest.trim().trim_matches(['"', '\'']) == schema)
    })
}
