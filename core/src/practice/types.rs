#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    pub started_at: Option<String>,
    pub spent_sec: u32,
    pub expired: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Start,
    Pause,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timer {
    pub spent_sec: u32,
    pub left_sec: i64,
    pub running: bool,
    pub expired: bool,
}
