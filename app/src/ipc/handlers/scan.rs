use tolearn_core::progress::Format;

use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{Absent, Broken, ScanIn, ScanOut};

pub fn run(input: &ScanIn) -> Result<ScanOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    Ok(ScanOut {
        root: scan.root.display().to_string(),
        format: match scan.format {
            Format::Json => "json".to_owned(),
            Format::Yaml => "yaml".to_owned(),
        },
        topics: scan.topics.iter().map(|topic| topic.id.clone()).collect(),
        absent: scan
            .absent
            .iter()
            .map(|absent| Absent {
                id: absent.id.clone(),
                stage: absent.stage,
                generated: absent.generated,
                file: absent.file.clone(),
            })
            .collect(),
        broken: scan
            .broken
            .iter()
            .map(|broken| Broken {
                id: broken.id.clone(),
                file: broken.file.clone(),
                message: broken.error.to_string(),
            })
            .collect(),
    })
}
