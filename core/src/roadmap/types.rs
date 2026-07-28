use std::collections::BTreeMap;

use crate::Hours;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roadmap {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub subject: String,
    pub goal: String,
    pub generated_at: String,
    pub generated_by: String,
    pub locale: String,
    pub weekly_hours: u32,
    pub env_constraints: Vec<String>,
    pub version_pins: BTreeMap<String, String>,
    pub calibration: Calibration,
    pub defaults: Defaults,
    pub stages: Vec<Stage>,
    pub topics: Vec<TopicEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calibration {
    pub method: CalibrationMethod,
    pub probed: u32,
    pub passed_out: Vec<String>,
    pub interrupted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationMethod {
    DiagnosticProbe,
    SelfReport,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    pub revalidate_after_days: RevalidateAfterDays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevalidateAfterDays {
    pub stable: u32,
    pub evolving: u32,
    pub volatile: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub n: u32,
    pub title: String,
    pub generated: bool,
    pub checkpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicEntry {
    pub id: String,
    pub title: String,
    pub stage: u32,
    pub file: String,
    pub est_hours: Hours,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Core,
    Recommended,
    Optional,
}

impl Priority {
    pub fn label(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Recommended => "recommended",
            Self::Optional => "optional",
        }
    }
}
