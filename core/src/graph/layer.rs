use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Walking,
    Done(usize),
}

pub fn layers(order: &[String], edges: &HashMap<String, Vec<String>>) -> HashMap<String, usize> {
    let mut marks: HashMap<String, Mark> = HashMap::new();
    for id in order {
        walk(id, edges, &mut marks);
    }
    order
        .iter()
        .map(|id| {
            let layer = match marks.get(id) {
                Some(Mark::Done(layer)) => *layer,
                _ => 0,
            };
            (id.clone(), layer)
        })
        .collect()
}

fn walk(
    id: &str,
    edges: &HashMap<String, Vec<String>>,
    marks: &mut HashMap<String, Mark>,
) -> usize {
    match marks.get(id) {
        Some(Mark::Done(layer)) => return *layer,
        Some(Mark::Walking) => return 0,
        None => {}
    }
    marks.insert(id.to_owned(), Mark::Walking);
    let deep = edges
        .get(id)
        .into_iter()
        .flatten()
        .map(|dependency| walk(dependency, edges, marks) + 1)
        .max()
        .unwrap_or(0);
    marks.insert(id.to_owned(), Mark::Done(deep));
    deep
}
