#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "export gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;

use tolearn_core::Date;
use tolearn_core::export::markdown;
use tolearn_core::notes::{Note, Stamp};
use tolearn_core::status::effective;

use support::bundles;

const TODAY: &str = "2026-07-29";

fn day() -> Date {
    Date::parse(TODAY).unwrap()
}

fn note(topic: &str, body: &str) -> Note {
    Note {
        roadmap: "corpus-program".to_owned(),
        topic: topic.to_owned(),
        path: PathBuf::from(format!("{topic}.md")),
        body: body.to_owned(),
        stamp: Stamp {
            modified_nanos: 0,
            size: 0,
        },
    }
}

fn exported(passed: &[(&str, &str)], notes: &[Note]) -> String {
    let (map, topics) = bundles::corpus_whole();
    let state = bundles::recorded(passed);
    let statuses = effective(&map, &topics, &state, day());
    markdown(&map, &topics, &statuses, notes)
}

fn plain() -> String {
    exported(&[], &[])
}

fn anchors(text: &str) -> Vec<String> {
    let mut taken: Vec<String> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced || !line.starts_with('#') {
            continue;
        }
        let heading = line.trim_start_matches('#').trim();
        let base: String = heading
            .chars()
            .filter_map(|letter| {
                if letter.is_alphanumeric() || letter == '_' {
                    Some(letter.to_lowercase().to_string())
                } else if letter.is_whitespace() || letter == '-' {
                    Some("-".to_owned())
                } else {
                    None
                }
            })
            .collect();
        let base = base.trim_matches('-').to_owned();
        let count = seen.iter().filter(|known| **known == base).count();
        seen.push(base.clone());
        taken.push(if count == 0 {
            base
        } else {
            format!("{base}-{count}")
        });
    }
    taken
}

fn contents(text: &str) -> String {
    let start = text.find("## Оглавление").unwrap();
    let rest = &text[start..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    rest[..end].to_owned()
}

fn targets(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(place) = rest.find("](#") {
        rest = &rest[place + 3..];
        let end = rest.find(')').unwrap();
        found.push(rest[..end].to_owned());
    }
    found
}

#[test]
fn документ_начинается_с_названия_программы() {
    let text = plain();

    let first = text.lines().next().unwrap();

    assert!(first.starts_with("# "), "{first}");
    assert!(text.ends_with('\n'));
    assert!(!text.contains("\n\n\n"), "лишние пустые строки");
}

#[test]
fn каждая_тема_роадмапа_попала_в_документ() {
    let (map, _) = bundles::corpus_whole();
    let text = plain();

    for entry in &map.topics {
        assert!(
            text.contains(&format!("### {}", entry.title)),
            "{}",
            entry.id
        );
    }
}

#[test]
fn ссылки_оглавления_ведут_в_существующие_заголовки() {
    let text = plain();
    let known = anchors(&text);

    let wanted = targets(&text);

    assert!(!wanted.is_empty(), "в документе нет ни одной ссылки");
    for target in &wanted {
        assert!(known.contains(target), "ссылка `#{target}` никуда не ведёт");
    }
}

#[test]
fn зависимость_ведёт_ссылкой_в_свою_тему() {
    let (_, topics) = bundles::corpus_whole();
    let depends = topics
        .iter()
        .find(|topic| !topic.depends_on.is_empty())
        .unwrap();

    let text = plain();

    assert!(text.contains("Зависит от: ["), "{}", depends.id);
}

#[test]
fn ответы_экспортируются_только_для_зачтённых() {
    let (_, topics) = bundles::corpus_whole();
    let asked = topics
        .iter()
        .find(|topic| {
            topic
                .questions
                .iter()
                .any(|question| !question.expected_signals.is_empty())
        })
        .unwrap();
    let signal = asked.questions[0].expected_signals[0].clone();

    let closed = exported(&[], &[]);
    let open = exported(&[(&asked.id, "passed")], &[]);

    assert!(!closed.contains(&signal), "ответ утёк из незачтённой темы");
    assert!(open.contains(&signal), "зачтённая тема лишилась ответа");
}

#[test]
fn понижённая_тема_ответы_сохраняет() {
    let (_, topics) = bundles::corpus_whole();
    let asked = topics
        .iter()
        .find(|topic| !topic.exam.traps.is_empty())
        .unwrap();
    let trap = asked.exam.traps[0].clone();

    let open = exported(&[(&asked.id, "stale_passed")], &[]);

    assert!(open.contains(&trap), "ловушка пропала у понижённой темы");
}

#[test]
fn незачтённая_тема_говорит_что_ответы_скрыты() {
    let text = plain();

    assert!(text.contains("не зачтена"), "скрытие не объяснено");
}

#[test]
fn конспект_попадает_в_документ_а_его_заголовки_опускаются() {
    let (_, topics) = bundles::corpus_whole();
    let topic = topics[0].id.clone();

    let text = exported(&[], &[note(&topic, "# Мой заголовок\n\nтело конспекта")]);

    assert!(text.contains("#### Конспект"), "конспекта нет");
    assert!(text.contains("##### Мой заголовок"), "заголовок не опущен");
    assert!(text.contains("тело конспекта"));
}

#[test]
fn решётка_внутри_кода_за_заголовок_не_считается() {
    let (_, topics) = bundles::corpus_whole();
    let topic = topics[0].id.clone();

    let text = exported(&[], &[note(&topic, "```sh\n# комментарий\n```")]);

    assert!(text.contains("# комментарий"), "код переписан");
    assert!(
        !text.contains("##### комментарий"),
        "код принят за заголовок"
    );
}

#[test]
fn конспект_чужой_темы_в_раздел_не_попадает() {
    let text = exported(&[], &[note("нет-такой-темы", "чужое тело")]);

    assert!(!text.contains("чужое тело"));
    assert!(!text.contains("#### Конспект"));
}

#[test]
fn несгенерированная_тема_названа_а_не_пропущена() {
    let (map, topics) = bundles::corpus();
    let state = bundles::recorded(&[]);
    let statuses = effective(&map, &topics, &state, day());

    let text = markdown(&map, &topics, &statuses, &[]);

    let absent = map
        .topics
        .iter()
        .find(|entry| !topics.iter().any(|topic| topic.id == entry.id))
        .unwrap();
    assert!(
        text.contains(&format!("### {}", absent.title)),
        "тема пропала"
    );
    assert!(text.contains("ещё не сгенерирована"));
}

#[test]
fn материалы_выведены_ссылками() {
    let (_, topics) = bundles::corpus_whole();
    let material = topics
        .iter()
        .flat_map(|topic| topic.materials.iter())
        .next()
        .unwrap()
        .clone();

    let text = plain();

    assert!(
        text.contains(&format!("]({})", material.url)),
        "материал не стал ссылкой"
    );
}

#[test]
fn одинаковые_названия_тем_разводятся_якорями() {
    let text = plain();
    let known = anchors(&text);
    let mut sorted = known.clone();
    sorted.sort();
    sorted.dedup();

    assert_eq!(sorted.len(), known.len(), "якоря повторяются: {known:?}");
}

#[test]
fn этапы_идут_по_порядку_роадмапа() {
    let (map, _) = bundles::corpus_whole();
    let text = plain();

    let mut last = 0;
    for stage in &map.stages {
        let heading = format!("## Этап {} — {}", stage.n, stage.title);
        let place = text.find(&heading).unwrap_or_else(|| panic!("{heading}"));
        assert!(place > last, "этапы переставлены: {heading}");
        last = place;
    }
}

#[test]
fn практика_экспортируется_всегда() {
    let (_, topics) = bundles::corpus_whole();
    let task = topics[0].practice.task.clone();

    let text = plain();

    assert!(text.contains(&task), "практика скрыта вместе с ответами");
}

#[test]
fn голый_адрес_в_тексте_бандла_становится_автоссылкой() {
    let (map, topics) = bundles::reference();
    let state = bundles::recorded(&[]);
    let statuses = effective(&map, &topics, &state, day());

    let text = markdown(&map, &topics, &statuses, &[]);

    assert!(text.contains("<http://localhost:11434"), "адрес голый");
}

#[test]
fn конспект_переписан_не_будет() {
    let (_, topics) = bundles::corpus_whole();
    let topic = topics[0].id.clone();
    let body = "смотри http://example.org/док, там же";

    let text = exported(&[], &[note(&topic, body)]);

    assert!(text.contains(body), "конспект переписан");
}

#[test]
fn ссылка_материала_повторно_не_оборачивается() {
    let text = plain();

    assert!(!text.contains("](<http"), "ссылка испорчена обёрткой");
}

#[test]
fn адрес_внутри_кода_не_трогается() {
    let (_, topics) = bundles::corpus_whole();
    let topic = topics[0].id.clone();

    let text = exported(&[], &[note(&topic, "`curl http://localhost:1234` вручную")]);

    assert!(
        text.contains("`curl http://localhost:1234`"),
        "код переписан"
    );
}

#[test]
fn ловушки_незачтённой_темы_наружу_не_выходят() {
    let (_, topics) = bundles::corpus_whole();
    let asked = topics
        .iter()
        .find(|topic| !topic.exam.traps.is_empty())
        .unwrap();
    let trap = asked.exam.traps[0].clone();

    let text = plain();

    assert!(!text.contains(&trap), "ловушка утекла из незачтённой темы");
}

#[test]
fn две_темы_с_одним_названием_получают_разные_якоря() {
    let (mut map, topics) = bundles::corpus_whole();
    let title = map.topics[0].title.clone();
    map.topics[1].title = title.clone();
    let state = bundles::recorded(&[]);
    let statuses = effective(&map, &topics, &state, day());

    let text = markdown(&map, &topics, &statuses, &[]);

    let known = anchors(&text);
    let listed = targets(&contents(&text));
    let mut sorted = listed.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        listed.len(),
        "оглавление ведёт дважды в одно место: {listed:?}"
    );
    for target in &listed {
        assert!(known.contains(target), "ссылка `#{target}` никуда не ведёт");
    }
}

#[test]
fn дефис_в_названии_остаётся_в_якоре() {
    let (mut map, topics) = bundles::corpus_whole();
    map.topics[0].title = "Alpha-beta гамма".to_owned();
    let state = bundles::recorded(&[]);
    let statuses = effective(&map, &topics, &state, day());

    let text = markdown(&map, &topics, &statuses, &[]);

    assert!(text.contains("(#alpha-beta-гамма)"), "якорь потерял дефис");
}

#[test]
fn адрес_в_коде_внутри_бандла_остаётся_кодом() {
    let (map, mut topics) = bundles::corpus_whole();
    topics[0].practice.task = "запусти `curl http://localhost:1234` руками".to_owned();
    let state = bundles::recorded(&[]);
    let statuses = effective(&map, &topics, &state, day());

    let text = markdown(&map, &topics, &statuses, &[]);

    assert!(
        text.contains("`curl http://localhost:1234`"),
        "код переписан"
    );
    assert!(
        !text.contains("<http://localhost:1234>"),
        "код обёрнут ссылкой"
    );
}
