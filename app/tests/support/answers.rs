use super::generating::Case;
use super::repository;

pub const BROKEN: &str = "```yaml\nschema: learning-roadmap/v1\nid: [\n```\n";

pub fn read(path: &str) -> String {
    std::fs::read_to_string(repository().join(path)).unwrap()
}

pub fn skeleton() -> String {
    let map =
        read("fixtures/valid/roadmap/minimal.yaml").replace("generated: true", "generated: false");
    let progress = read("fixtures/valid/progress/minimal.yaml");
    format!("```yaml\n{map}```\n\nи прогресс:\n\n```yaml\n{progress}```\n")
}

pub fn topic() -> String {
    format!(
        "```yaml\n{}```\n",
        read("fixtures/valid/topic/minimal-topic.yaml")
    )
}

pub fn two() -> (String, String) {
    let map = format!(
        "{}  - id: second-topic\n    title: Вторая тема\n    stage: 1\n    \
         file: topics/second-topic.yaml\n    est_hours: [1, 2]\n    priority: core\n",
        read("fixtures/valid/roadmap/minimal.yaml").replace("generated: true", "generated: false")
    );
    let progress = format!(
        "{}  second-topic:\n    status: todo\n    attempts: []\n    passed_at: null\n    \
         next_review_at: null\n    gaps: []\n",
        read("fixtures/valid/progress/minimal.yaml")
    );
    (map, progress)
}

pub fn paired() -> String {
    let (map, progress) = two();
    format!("```yaml\n{map}```\n\nи прогресс:\n\n```yaml\n{progress}```\n")
}

pub fn looped(case: &Case) {
    let (map, progress) = two();
    let root = case.data.join("draft");
    std::fs::create_dir_all(root.join("topics")).unwrap();
    std::fs::write(root.join("roadmap.yaml"), map).unwrap();
    std::fs::write(root.join("progress.yaml"), progress).unwrap();
    std::fs::write(
        root.join("topics/minimal-topic.yaml"),
        written(
            "minimal-topic",
            "Минимальная тема",
            "depends_on:\n  - second-topic",
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("topics/second-topic.yaml"),
        written(
            "second-topic",
            "Вторая тема",
            "depends_on:\n  - minimal-topic",
        ),
    )
    .unwrap();
}

pub fn written(id: &str, title: &str, depends: &str) -> String {
    read("fixtures/valid/topic/minimal-topic.yaml")
        .replace("id: minimal-topic", &format!("id: {id}"))
        .replace("title: Минимальная тема", &format!("title: {title}"))
        .replace("depends_on: []", depends)
}

pub fn answered(id: &str, title: &str, depends: &str) -> String {
    format!("```yaml\n{}```\n", written(id, title, depends))
}
