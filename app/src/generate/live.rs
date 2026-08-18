use super::made::Summary;

const TAIL_CHARS: usize = 600;

#[derive(Debug, Clone, Default)]
pub struct Live {
    pub step: String,
    pub total: usize,
    pub done: usize,
    pub attempt: u32,
    pub retry: u32,
    pub current: String,
    pub waiting: bool,
    pub finished: bool,
    pub cancelled: bool,
    pub refused: Vec<String>,
    pub missed: Vec<String>,
    pub seconds: u64,
    pub step_seconds: u64,
    pub chars: u64,
    pub ticks: u64,
    pub tail: String,
    pub tokens: Option<u32>,
    pub summary: Option<Summary>,
}

pub fn tailed(tail: &str, piece: &str) -> String {
    let whole = format!("{tail}{piece}");
    let extra = whole.chars().count().saturating_sub(TAIL_CHARS);
    whole.chars().skip(extra).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn хвост_не_растёт_дальше_потолка() {
        let long = "я".repeat(TAIL_CHARS * 2);

        let tail = tailed("начало\n", &long);

        assert_eq!(tail.chars().count(), TAIL_CHARS);
        assert!(!tail.contains("начало"), "{tail}");
    }

    #[test]
    fn короткий_хвост_копится_целиком() {
        assert_eq!(tailed("раз\n", "два"), "раз\nдва");
    }
}
