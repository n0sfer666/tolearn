pub fn words(text: &str) -> Vec<String> {
    text.split(|letter: char| !letter.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(spelled)
        .collect()
}

fn spelled(word: &str) -> String {
    word.to_lowercase().replace('ё', "е")
}
