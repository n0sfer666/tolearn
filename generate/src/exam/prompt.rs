use tolearn_core::verdict::VerdictError;

use super::paper::Paper;

pub fn prompt(paper: &Paper<'_>) -> String {
    let stage = paper.stage;
    let questions: Vec<String> = stage
        .questions
        .iter()
        .map(|question| {
            let answer = paper
                .answers
                .get(&question.id)
                .map(|text| text.trim())
                .filter(|text| !text.is_empty())
                .unwrap_or("нет ответа");
            format!(
                "Вопрос {}: {}\nЭталон: {}\nОтвет ученика:\n{answer}",
                question.id, question.text, question.answer
            )
        })
        .collect();
    let format = format!(
        r#"{{"stage": "{}", "per_question": [{{"id": "q1", "result": "ok"}}, {{"id": "q2", "result": "partial", "missed": ["что упущено"]}}]}}"#,
        stage.id
    );
    format!(
        "Ты принимаешь письменный зачёт по этапу учебной программы «{}» (id этапа: {}).\n\n\
         Уровень ученика: {}\n\
         Язык программы: {} — на нём пиши missed.\n\n\
         Сверь каждый ответ ученика с эталоном и поставь оценку:\n\
         - ok — ответ передаёт суть эталона, пусть и своими словами;\n\
         - partial — суть схвачена не целиком: важной части эталона нет или в ответе ошибка;\n\
         - miss — ответ пуст или мимо сути.\n\n\
         Правила:\n\
         - Оценивай только написанное: не додумывай за ученика и не засчитывай то, чего в ответе нет.\n\
         - Не хвали и не смягчай оценку: вежливость — не повод поднять её.\n\
         - У partial и miss перечисли в missed, что именно упущено, по пункту на строку. У ok missed не нужен.\n\
         - Оцени каждый вопрос ровно один раз, id бери из заголовков вопросов.\n\n\
         {}\n\n\
         Можешь сначала коротко разобрать ответы, но закончи ответ блоком JSON:\n{format}",
        stage.title,
        stage.id,
        paper.level,
        paper.locale,
        questions.join("\n\n")
    )
}

pub(super) fn repair(task: &str, answer: &str, error: &VerdictError) -> String {
    format!(
        "{task}\n\n\
         Твой прошлый ответ:\n{answer}\n\n\
         Вердикт из него не разобран: {error}\n\n\
         Пришли разбор заново и закончи его блоком JSON в том же формате, по строке на каждый вопрос."
    )
}
