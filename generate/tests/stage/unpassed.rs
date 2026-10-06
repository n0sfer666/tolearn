use tolearn_core::state::Lapse;
use tolearn_generate::unpassed::{UNPASSED_CHARS, block};

const HEAD: &str = "Незачтённые вопросы — на зачёте ученик ответил на них не полностью:";

fn lapse(stage: &str, question: &str, missed: &[&str]) -> Lapse {
    Lapse {
        stage: stage.to_owned(),
        question: question.to_owned(),
        missed: missed.iter().map(|&part| part.to_owned()).collect(),
    }
}

#[test]
fn without_lapses_there_is_no_block() {
    assert_eq!(block(&[]), None);
}

#[test]
fn the_block_lists_each_question_with_what_was_missed() {
    let lapses = [
        lapse("Трекер", "Что такое трекер?", &["строки", "каналы"]),
        lapse("Голоса чипа", "Сколько голосов у чипа?", &[]),
    ];

    assert_eq!(
        block(&lapses).unwrap(),
        format!(
            "{HEAD}\n\
             - «Трекер»: Что такое трекер?\n  Упущено: строки; каналы\n\
             - «Голоса чипа»: Сколько голосов у чипа?"
        )
    );
}

#[test]
fn an_overflowing_block_keeps_the_latest_questions_within_the_limit() {
    let lapses: Vec<Lapse> = (0..100)
        .map(|index| {
            lapse(
                "Этап",
                &format!("Вопрос номер {index} про звук старых приставок?"),
                &["громкость"],
            )
        })
        .collect();

    let block = block(&lapses).unwrap();

    assert!(block.chars().count() <= UNPASSED_CHARS);
    assert!(block.starts_with(HEAD));
    assert!(block.contains("Вопрос номер 99 "));
    assert!(!block.contains("Вопрос номер 0 "));
}

#[test]
fn a_single_huge_question_is_cut_to_the_limit() {
    let block = block(&[lapse("Этап", &"я".repeat(5_000), &["всё"])]).unwrap();

    assert_eq!(block.chars().count(), UNPASSED_CHARS);
    assert!(block.starts_with(&format!("{HEAD}\n- «Этап»: я")));
}

#[test]
fn the_block_limit_is_pinned() {
    assert_eq!(UNPASSED_CHARS, 1_500);
}
