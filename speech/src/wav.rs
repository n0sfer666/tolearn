use std::path::Path;

use crate::error::SpeechError;
use crate::pcm::{Pcm, RATE};

const PCM: u16 = 1;
const BITS: u16 = 16;
const MONO: u16 = 1;

pub fn read(path: &Path) -> Result<Pcm, SpeechError> {
    let bytes = std::fs::read(path).map_err(|error| SpeechError::Unreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    decode(&bytes)
}

pub fn decode(bytes: &[u8]) -> Result<Pcm, SpeechError> {
    if tag(bytes, 0) != Some(b"RIFF") || tag(bytes, 8) != Some(b"WAVE") {
        return Err(SpeechError::Unsupported("это не WAV".to_owned()));
    }
    let mut format = None;
    let mut at = 12;
    while let (Some(id), Some(size)) = (tag(bytes, at), word(bytes, at + 4)) {
        let body = at + 8;
        let size = size as usize;
        let Some(chunk) = bytes.get(body..body.saturating_add(size)) else {
            break;
        };
        match id {
            b"fmt " => format = Some(shape(chunk)?),
            b"data" if format.is_some() => return samples(chunk),
            _ => {}
        }
        at = body + size + usize::from(!size.is_multiple_of(2));
    }
    Err(SpeechError::Unsupported(
        "в файле нет пары «fmt » и «data»".to_owned(),
    ))
}

fn shape(chunk: &[u8]) -> Result<(), SpeechError> {
    let (Some(kind), Some(channels), Some(rate), Some(bits)) = (
        half(chunk, 0),
        half(chunk, 2),
        word(chunk, 4),
        half(chunk, 14),
    ) else {
        return Err(SpeechError::Unsupported("заголовок оборван".to_owned()));
    };
    if kind != PCM || bits != BITS {
        return Err(SpeechError::Unsupported(format!(
            "ждём несжатый PCM {BITS} бит, у файла — формат {kind}, {bits} бит"
        )));
    }
    if channels != MONO {
        return Err(SpeechError::Unsupported(format!(
            "ждём моно, у файла — каналов: {channels}"
        )));
    }
    if rate != RATE {
        return Err(SpeechError::Unsupported(format!(
            "ждём {RATE} Гц, у файла — {rate} Гц"
        )));
    }
    Ok(())
}

fn samples(chunk: &[u8]) -> Result<Pcm, SpeechError> {
    if !chunk.len().is_multiple_of(2) {
        return Err(SpeechError::Unsupported(
            "звуковые данные оборваны на середине отсчёта".to_owned(),
        ));
    }
    Ok(Pcm::new(
        chunk
            .chunks_exact(2)
            .map(|pair| f32::from(i16::from_le_bytes([pair[0], pair[1]])) / 32_768.0)
            .collect(),
    ))
}

fn tag(bytes: &[u8], at: usize) -> Option<&[u8]> {
    bytes.get(at..at + 4)
}

fn word(bytes: &[u8], at: usize) -> Option<u32> {
    tag(bytes, at).map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
}

fn half(bytes: &[u8], at: usize) -> Option<u16> {
    bytes
        .get(at..at + 2)
        .map(|two| u16::from_le_bytes([two[0], two[1]]))
}
