use std::iter::Sum;
use std::ops::AddAssign;

use crate::words::words;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Score {
    pub errors: usize,
    pub words: usize,
}

impl Score {
    pub fn rate(&self) -> f64 {
        if self.words == 0 {
            return if self.errors == 0 { 0.0 } else { 1.0 };
        }
        self.errors as f64 / self.words as f64
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        self.errors += other.errors;
        self.words += other.words;
    }
}

impl Sum for Score {
    fn sum<I: Iterator<Item = Self>>(scores: I) -> Self {
        scores.fold(Self::default(), |mut total, score| {
            total += score;
            total
        })
    }
}

pub fn score(reference: &str, heard: &str) -> Score {
    let reference = words(reference);
    let heard = words(heard);
    Score {
        errors: distance(&reference, &heard),
        words: reference.len(),
    }
}

fn distance(reference: &[String], heard: &[String]) -> usize {
    let mut row: Vec<usize> = (0..=heard.len()).collect();
    for (above, word) in reference.iter().enumerate() {
        let mut corner = row[0];
        row[0] = above + 1;
        for (left, said) in heard.iter().enumerate() {
            let kept = corner + usize::from(word != said);
            corner = row[left + 1];
            row[left + 1] = kept.min(corner + 1).min(row[left] + 1);
        }
    }
    row[heard.len()]
}
