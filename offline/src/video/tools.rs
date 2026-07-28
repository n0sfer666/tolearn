use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Ytdlp,
    Ffmpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Macos,
    Linux,
    Windows,
}

#[derive(Debug, Clone)]
pub struct Absent {
    pub tool: Tool,
    pub install: String,
}

#[derive(Debug, Clone)]
pub struct Tools {
    pub ytdlp: PathBuf,
    pub ffmpeg: PathBuf,
}

impl Tool {
    pub fn binary(self) -> &'static str {
        match self {
            Self::Ytdlp => "yt-dlp",
            Self::Ffmpeg => "ffmpeg",
        }
    }

    pub fn install(self, os: Os) -> String {
        let binary = self.binary();
        match os {
            Os::Macos => format!("brew install {binary}"),
            Os::Linux => format!("sudo apt install {binary}"),
            Os::Windows => format!("winget install {binary}"),
        }
    }
}

impl Os {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

pub fn ready(path: &str) -> Result<Tools, Vec<Absent>> {
    let here = Os::current();
    match (look(Tool::Ytdlp, path), look(Tool::Ffmpeg, path)) {
        (Some(ytdlp), Some(ffmpeg)) => Ok(Tools { ytdlp, ffmpeg }),
        (ytdlp, ffmpeg) => Err([
            (Tool::Ytdlp, ytdlp.is_none()),
            (Tool::Ffmpeg, ffmpeg.is_none()),
        ]
        .into_iter()
        .filter(|(_, absent)| *absent)
        .map(|(tool, _)| Absent {
            tool,
            install: tool.install(here),
        })
        .collect()),
    }
}

fn look(tool: Tool, path: &str) -> Option<PathBuf> {
    path.split(':')
        .filter(|part| !part.is_empty())
        .map(|part| PathBuf::from(part).join(tool.binary()))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn пустой_путь_ничего_не_находит() {
        assert!(look(Tool::Ytdlp, "").is_none());
        assert!(look(Tool::Ytdlp, "::").is_none());
    }

    #[test]
    fn команда_установки_называет_инструмент() {
        assert!(Tool::Ffmpeg.install(Os::Linux).ends_with("ffmpeg"));
    }
}
