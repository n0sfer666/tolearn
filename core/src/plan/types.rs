use crate::Hours;

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub weekly_hours: u32,
    pub daily_hours: f64,
    pub left: Hours,
    pub unknown: u32,
    pub soonest: Ahead,
    pub latest: Ahead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ahead {
    pub days: u32,
    pub date: String,
}
