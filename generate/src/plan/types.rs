use tolearn_core::Hours;
use tolearn_core::program::{StageRow, Volatility};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub request: String,
    pub level: String,
    pub locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub title: String,
    pub slug: String,
    pub goal: String,
    pub volatility: Volatility,
    pub stages: Vec<StageRow>,
    pub children: Vec<Part>,
}

impl Plan {
    pub fn hours(&self) -> Hours {
        self.stages
            .iter()
            .map(|row| row.hours)
            .chain(self.children.iter().map(|row| row.hours))
            .fold(Hours::default(), |sum, hours| Hours {
                min: sum.min.saturating_add(hours.min),
                max: sum.max.saturating_add(hours.max),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub title: String,
    pub goal: String,
    pub hours: Hours,
}
