#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, Shelf};
use tolearn_core::state::{Answered, Attempt, Grade, Sitting, State};

fn stage(shelf: &Shelf) -> Value {
    shelf
        .ask(
            "stage",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices" }),
        )
        .unwrap()
}

fn row(id: &str, result: Grade, missed: &[&str]) -> Answered {
    Answered {
        id: id.to_owned(),
        result,
        missed: missed.iter().map(|&line| line.to_owned()).collect(),
    }
}

fn sat(shelf: &Shelf, on: &str, per_question: Vec<Answered>) {
    State::update(&shelf.data, CHIPTUNE, |state| {
        state.attempt(
            CHIPTUNE,
            "voices",
            Attempt {
                on: on.to_owned(),
                by: Sitting::Copypaste,
                model: None,
                per_question,
            },
        );
    })
    .unwrap();
}

#[test]
fn незачтённое_последней_попытки_стоит_у_своих_вопросов() {
    let shelf = Shelf::new("graded");
    shelf.shelved("examples/chiptune");

    let fresh = stage(&shelf);
    for question in fresh["questions"].as_array().unwrap() {
        assert_eq!(question["result"], Value::Null, "{question}");
        assert_eq!(question["missed"], json!([]), "{question}");
    }

    sat(
        &shelf,
        "2026-09-12",
        vec![
            row("q1", Grade::Miss, &["старое"]),
            row("q2", Grade::Ok, &[]),
            row("q3", Grade::Ok, &[]),
            row("q4", Grade::Ok, &[]),
        ],
    );
    sat(
        &shelf,
        "2026-09-13",
        vec![
            row("q1", Grade::Ok, &[]),
            row(
                "q2",
                Grade::Partial,
                &["почему 25 % и 75 % звучат одинаково"],
            ),
            row("q3", Grade::Ok, &[]),
            row("q4", Grade::Miss, &["формула таймера", "частота NTSC"]),
        ],
    );

    let questions = stage(&shelf)["questions"].clone();
    let seen = |at: usize| {
        (
            questions[at]["result"].clone(),
            questions[at]["missed"].clone(),
        )
    };
    assert_eq!(seen(0), (json!("ok"), json!([])));
    assert_eq!(
        seen(1),
        (
            json!("partial"),
            json!(["почему 25 % и 75 % звучат одинаково"])
        )
    );
    assert_eq!(
        seen(3),
        (json!("miss"), json!(["формула таймера", "частота NTSC"]))
    );
}

#[test]
fn эталон_приходит_только_у_вопроса_с_вердиктом() {
    let shelf = Shelf::new("mirror");
    shelf.shelved("examples/chiptune");

    let fresh = stage(&shelf);
    for question in fresh["questions"].as_array().unwrap() {
        assert_eq!(question["answer"], Value::Null, "{question}");
    }

    sat(
        &shelf,
        "2026-09-14",
        vec![
            row("q1", Grade::Ok, &[]),
            row("q2", Grade::Miss, &["весь ответ"]),
        ],
    );

    let after = stage(&shelf);
    let questions = after["questions"].as_array().unwrap();
    let answered = |id: &str| {
        questions
            .iter()
            .find(|question| question["id"] == json!(id))
            .unwrap()["answer"]
            .clone()
    };
    assert!(
        answered("q1").as_str().unwrap().contains("Два импульсных"),
        "{}",
        answered("q1")
    );
    assert!(
        answered("q2").as_str().unwrap().contains("перевёрнутая"),
        "{}",
        answered("q2")
    );
    assert_eq!(answered("q3"), Value::Null);
}
