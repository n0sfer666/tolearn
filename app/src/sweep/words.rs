pub const OVER: &str = "Вопросы кончились. Можно завершать прогон — или добавить, если есть что.";

pub fn opening(topics: usize, questions: usize) -> String {
    format!(
        "Сквозной зачёт по программе. Тем в прогоне: {topics}, вопросов: {questions}. \
         Вопросы идут вперемешку — по одному из каждой темы. Подсказки снижают итог."
    )
}

pub fn question(title: &str, text: &str) -> String {
    format!("Тема «{title}».\n\n{text}")
}
