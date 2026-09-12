use serde_json::json;

use crate::answers::{CALLOUT, DELIVERABLE, LONG, QUESTION, compose, drafted, fine, ids};
use crate::web::chiptune;

#[test]
fn a_broken_block_goes_to_repair_alone_and_only_its_id_changes() {
    let stage = fine();
    let mut broken = stage.clone();
    broken["blocks"][1]["sources"] = json!(["p9"]);
    let before = ids(&broken);
    let text = "У чипа NES пять каналов, подробно — на https://a.test/article.";
    let mended = json!({"blocks": {before[1].clone(): {"kind": "paragraph", "text": text, "sources": ["p1"]}}});
    let run = compose("mend-block", &chiptune(), "voices", &[broken, mended]);

    let after = drafted(&run.result.unwrap());
    assert_eq!(run.prompts.len(), 2);
    let repair = &run.prompts[1];
    assert!(repair.contains(&format!("\"{}\"", before[1])));
    assert!(repair.contains("p9"));
    for whole in [CALLOUT, DELIVERABLE, QUESTION, LONG, "Извлечённый текст"] {
        assert!(!repair.contains(whole), "{whole}");
    }
    assert_ne!(after[1], before[1]);
    let kept: Vec<_> = (0..before.len()).filter(|index| *index != 1).collect();
    for index in kept {
        assert_eq!(after[index], before[index]);
    }
}

#[test]
fn a_stage_wide_flaw_resends_the_theory_whole_and_nothing_else() {
    let stage = fine();
    let mut broken = stage.clone();
    broken["blocks"].as_array_mut().unwrap().pop();
    broken["blocks"][1]["sources"] = json!(["p9"]);
    broken["terms"] = json!((1..=9).map(|n| format!("термин {n}")).collect::<Vec<_>>());
    let mended = json!({"theory": stage["blocks"].clone(), "terms": stage["terms"].clone()});
    let run = compose("mend-theory", &chiptune(), "voices", &[broken, mended]);

    assert_eq!(drafted(&run.result.unwrap()), ids(&stage));
    let repair = &run.prompts[1];
    assert!(repair.contains("\"theory\""));
    assert!(repair.contains("\"terms\""));
    assert!(repair.contains("текст теории"));
    assert!(repair.contains("9 новых терминов"));
    for whole in [
        "\"blocks\":{",
        "\"practice\"",
        "\"questions\"",
        DELIVERABLE,
        QUESTION,
    ] {
        assert!(!repair.contains(whole), "{whole}");
    }
}

#[test]
fn practice_and_questions_are_resent_without_the_theory() {
    let stage = fine();
    let mut broken = stage.clone();
    broken["practice"]["acceptance"] = json!([]);
    broken["questions"][1]["answer"] = json!("");
    let mended =
        json!({"practice": stage["practice"].clone(), "questions": stage["questions"].clone()});
    let run = compose("mend-practice", &chiptune(), "voices", &[broken, mended]);

    assert_eq!(drafted(&run.result.unwrap()), ids(&stage));
    let repair = &run.prompts[1];
    assert!(repair.contains("\"practice\""));
    assert!(repair.contains("\"questions\""));
    assert!(repair.contains("acceptance пуст"));
    assert!(repair.contains("у q2 нет эталонного ответа"));
    for whole in ["\"theory\"", CALLOUT, LONG] {
        assert!(!repair.contains(whole), "{whole}");
    }
}

#[test]
fn the_first_stage_takes_exactly_one_tool() {
    let stage = fine();
    for (label, tools) in [
        ("mend-no-tool", json!([])),
        ("mend-two-tools", json!(["FamiTracker", "Audacity"])),
    ] {
        let mut broken = stage.clone();
        broken["tools"] = tools.clone();
        let mended = json!({"practice": stage["practice"].clone(), "tools": ["FamiTracker"]});
        let run = compose(label, &chiptune(), "voices", &[broken, mended]);

        assert_eq!(drafted(&run.result.unwrap()), ids(&stage));
        let repair = &run.prompts[1];
        assert!(repair.contains("ровно 1 инструмент"), "{label}");
        assert!(repair.contains(&format!("\"tools\":{tools}")), "{label}");
        assert!(!repair.contains("\"theory\""), "{label}");
    }

    let mut later = stage;
    later["tools"] = json!(["FamiTracker", "Audacity"]);
    let run = compose("mend-later-tools", &chiptune(), "envelope", &[later]);
    assert!(run.result.is_ok());
    assert_eq!(run.prompts.len(), 1);
}
