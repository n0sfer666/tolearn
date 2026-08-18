use std::collections::HashMap;

use tolearn_core::topic::Material;

use super::piece::Piece;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mark {
    pub title: String,
    pub topic: String,
    pub topic_at: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Marks {
    topics: usize,
    at: HashMap<String, Mark>,
}

impl Marks {
    pub fn topics(&self) -> usize {
        self.topics
    }

    pub fn of(&self, url: &str) -> Mark {
        self.at.get(url).cloned().unwrap_or_default()
    }

    pub fn title(&self, url: &str) -> String {
        match self.at.get(url) {
            Some(mark) => mark.title.clone(),
            None => url.to_owned(),
        }
    }
}

pub fn split(pieces: Vec<Piece>) -> (Vec<Material>, Marks) {
    let mut order: Vec<String> = Vec::new();
    let mut at = HashMap::new();
    let mut materials = Vec::with_capacity(pieces.len());
    for piece in pieces {
        let topic_at = match order.iter().position(|topic| topic == &piece.topic) {
            Some(seen) => seen + 1,
            None => {
                order.push(piece.topic.clone());
                order.len()
            }
        };
        at.insert(
            piece.material.url.clone(),
            Mark {
                title: piece.material.title.clone(),
                topic: piece.topic,
                topic_at,
            },
        );
        materials.push(piece.material);
    }
    let marks = Marks {
        topics: order.len(),
        at,
    };
    (materials, marks)
}

#[cfg(test)]
mod tests {
    use tolearn_core::topic::{Liveness, MaterialTier, MaterialType};

    use super::*;

    fn piece(topic: &str, title: &str, url: &str) -> Piece {
        Piece {
            topic: topic.to_owned(),
            material: Material {
                title: title.to_owned(),
                url: url.to_owned(),
                kind: MaterialType::Docs,
                tier: MaterialTier::T1,
                lang: "ru".to_owned(),
                liveness: Liveness::Ok,
                published: None,
                covers_version: None,
                checked_at: "2026-07-27".to_owned(),
                stale: false,
                delta: None,
                note: String::new(),
            },
        }
    }

    #[test]
    fn темы_нумеруются_по_порядку_появления() {
        let (materials, marks) = split(vec![
            piece("Первая", "Гайд", "https://a.test/1"),
            piece("Первая", "Спека", "https://a.test/2"),
            piece("Вторая", "Статья", "https://a.test/3"),
        ]);

        assert_eq!(materials.len(), 3);
        assert_eq!(marks.topics(), 2);
        assert_eq!(marks.of("https://a.test/2").topic_at, 1);
        assert_eq!(marks.of("https://a.test/3").topic_at, 2);
        assert_eq!(marks.of("https://a.test/3").topic, "Вторая");
    }

    #[test]
    fn неизвестный_адрес_зовётся_собой() {
        let (_, marks) = split(vec![piece("Первая", "Гайд", "https://a.test/1")]);

        assert_eq!(marks.title("https://a.test/1"), "Гайд");
        assert_eq!(marks.title("https://a.test/нет"), "https://a.test/нет");
    }
}
