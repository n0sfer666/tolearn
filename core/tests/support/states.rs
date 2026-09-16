use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tolearn_core::state::{
    Answered, Attempt, Clarification, Grade, Pass, Passed, Sitting, StageState, State, Turn, key,
};

pub const PROGRAM: &str = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";

pub fn scratch(name: &str) -> PathBuf {
    super::scratch::made(&format!("state-{name}"))
}

pub fn file(data: &Path) -> PathBuf {
    data.join("state").join(PROGRAM).join("state.yaml")
}

pub fn placed(data: &Path, body: &str) -> PathBuf {
    let file = file(data);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, body).unwrap();
    file
}

pub fn full() -> State {
    let mut state = State::new(PROGRAM);
    state.workdir = Some(owned("/Users/learner/practice/chiptune"));
    state.stages.insert(key(PROGRAM, "voices"), voices());
    state.stages.insert(key(PROGRAM, "envelope"), envelope());
    state.clarifications = vec![
        Clarification {
            stage: key(PROGRAM, "voices"),
            block: owned("1a2b3c4d"),
            excerpt: owned("Звуковой чип NES"),
            fragment: Some(owned("скважность")),
            turns: vec![
                Turn {
                    asked: Some(owned("Что такое скважность?")),
                    answer: owned("Доля периода, когда сигнал высокий."),
                },
                Turn {
                    asked: None,
                    answer: owned("Иначе: сколько времени из такта звучит «верх»."),
                },
            ],
            clear: true,
        },
        Clarification {
            stage: key(PROGRAM, "envelope"),
            block: owned("5e6f7a8b-2"),
            excerpt: owned("Громкость спадает"),
            fragment: None,
            turns: vec![Turn {
                asked: None,
                answer: owned("Каждый кадр счётчик уменьшает громкость на единицу."),
            }],
            clear: false,
        },
    ];
    state
}

fn voices() -> StageState {
    StageState {
        opened: Some(owned("2026-09-13")),
        passed: Some(Passed {
            on: owned("2026-09-14"),
            by: Pass::Exam,
        }),
        ticks: vec![owned("c1"), owned("a1")],
        answers: BTreeMap::from([
            (
                owned("q1"),
                owned("Два импульсных канала,\nтреугольник и шум."),
            ),
            (owned("q2"), owned("Огибающая.")),
        ]),
        attempts: vec![Attempt {
            on: owned("2026-09-14"),
            by: Sitting::Written,
            model: Some(owned("claude-opus-5")),
            per_question: vec![
                Answered {
                    id: owned("q1"),
                    result: Grade::Partial,
                    missed: vec![owned("канал DPCM")],
                },
                Answered {
                    id: owned("q2"),
                    result: Grade::Ok,
                    missed: Vec::new(),
                },
            ],
        }],
        since: 0,
    }
}

fn envelope() -> StageState {
    StageState {
        opened: Some(owned("2026-09-15")),
        passed: Some(Passed {
            on: owned("2026-09-16"),
            by: Pass::Skip,
        }),
        attempts: vec![Attempt {
            on: owned("2026-09-16"),
            by: Sitting::Copypaste,
            model: None,
            per_question: vec![Answered {
                id: owned("q1"),
                result: Grade::Miss,
                missed: vec![owned("весь ответ")],
            }],
        }],
        since: 1,
        ..StageState::default()
    }
}

fn owned(text: &str) -> String {
    text.to_owned()
}
