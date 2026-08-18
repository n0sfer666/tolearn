pub fn tongue(locale: &str) -> String {
    let head = locale
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_lowercase();
    if head.len() == 2 && head.chars().all(|letter| letter.is_ascii_lowercase()) {
        head
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::tongue;

    #[test]
    fn берёт_язык_из_локали() {
        assert_eq!(tongue("ru"), "ru");
        assert_eq!(tongue("en-US"), "en");
        assert_eq!(tongue("pt_BR"), "pt");
        assert_eq!(tongue("RU"), "ru");
    }

    #[test]
    fn непонятная_локаль_отдаётся_на_автоопределение() {
        assert_eq!(tongue(""), "");
        assert_eq!(tongue("русский"), "");
        assert_eq!(tongue("r1"), "");
    }
}
