use crate::Hours;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub uuid: String,
    pub slug: String,
    pub title: String,
    pub goal: String,
    pub level: String,
    pub generation: Generation,
    pub map: Map,
    pub sources: Sources,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generation {
    pub locale: String,
    pub volatility: Volatility,
    pub request: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Volatility {
    Stable,
    Evolving,
    Volatile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub stages: Vec<StageRow>,
    pub children: Vec<ChildRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageRow {
    pub id: String,
    pub title: String,
    pub hours: Hours,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildRow {
    pub uuid: String,
    pub title: String,
    pub hours: Hours,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sources {
    pub books: Vec<Book>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    pub title: String,
    pub authors: Vec<String>,
    pub isbn: String,
    pub chapter: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub title: String,
    pub url: String,
    pub checked_at: String,
}
