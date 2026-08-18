#[derive(Debug, Clone, PartialEq, Default)]
pub struct Stats {
    pub attempts: u32,
    pub enough: bool,
    pub hinted: u32,
    pub hinted_share: f64,
    pub kinds: Vec<KindTally>,
    pub actions: Vec<ActionTally>,
    pub streak: Streak,
    pub calibration: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindTally {
    pub kind: String,
    pub ok: u32,
    pub partial: u32,
    pub miss: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionTally {
    pub action: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Streak {
    pub longest: u32,
    pub topic: String,
}
