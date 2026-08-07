use crate::types::{DEFAULT_TIMEOUT_SECS, Harness};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    pub id: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
}

impl Preset {
    pub fn harness(&self) -> Harness {
        Harness {
            id: self.id.to_owned(),
            command: self.command.to_owned(),
            args: self.args.iter().map(|arg| (*arg).to_owned()).collect(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

pub const CLAUDE: Preset = Preset {
    id: "claude",
    command: "claude",
    args: &[
        "-p",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--allowedTools",
        "",
    ],
};

pub const PRESETS: [Preset; 4] = [
    CLAUDE,
    Preset {
        id: "opencode",
        command: "opencode",
        args: &["run"],
    },
    Preset {
        id: "pi",
        command: "pi",
        args: &["-p", "--no-tools", "--no-session"],
    },
    Preset {
        id: "custom",
        command: "",
        args: &[],
    },
];

pub fn preset(id: &str) -> Option<Preset> {
    PRESETS.into_iter().find(|preset| preset.id == id)
}
