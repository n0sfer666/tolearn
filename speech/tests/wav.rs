#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "speech gate: a panic here is the report"
)]

use tolearn_speech::{Pcm, SpeechError, wav};

fn wave(channels: u16, rate: u32, bits: u16, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * u32::from(channels * bits / 8)).to_le_bytes());
    out.extend_from_slice(&(channels * bits / 8).to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    out
}

fn mono(samples: &[i16]) -> Vec<u8> {
    let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    wave(1, 16_000, 16, &data)
}

#[test]
fn отсчёты_разбираются_в_дробные_числа() {
    let heard = wav::decode(&mono(&[0, 16_384, -32_768, 32_767])).unwrap();

    assert_eq!(heard.samples().len(), 4);
    assert!((heard.samples()[0] - 0.0).abs() < f32::EPSILON);
    assert!((heard.samples()[1] - 0.5).abs() < f32::EPSILON);
    assert!((heard.samples()[2] + 1.0).abs() < f32::EPSILON);
    assert!(heard.samples()[3] < 1.0);
}

#[test]
fn длительность_считается_по_частоте_дискретизации() {
    let second: Vec<i16> = vec![0; 16_000];

    assert!((wav::decode(&mono(&second)).unwrap().seconds() - 1.0).abs() < 1e-9);
    assert_eq!(Pcm::default().seconds(), 0.0);
}

#[test]
fn чужой_формат_называется_а_не_молчит() {
    let stereo = wav::decode(&wave(2, 16_000, 16, &[0, 0, 0, 0]));
    let fast = wav::decode(&wave(1, 44_100, 16, &[0, 0]));
    let wide = wav::decode(&wave(1, 16_000, 32, &[0, 0, 0, 0]));

    for (heard, marker) in [(stereo, "каналов"), (fast, "44100"), (wide, "32 бит")] {
        match heard {
            Err(SpeechError::Unsupported(reason)) => {
                assert!(reason.contains(marker), "{reason}");
            }
            other => panic!("формат принят: {other:?}"),
        }
    }
}

#[test]
fn не_wav_отвергается_целиком() {
    assert!(matches!(
        wav::decode(b"\x89PNG\r\n\x1a\n"),
        Err(SpeechError::Unsupported(_))
    ));
    assert!(matches!(
        wav::decode(&wave(1, 16_000, 16, &[0])),
        Err(SpeechError::Unsupported(_))
    ));
}

#[test]
fn чужие_куски_файла_пропускаются() {
    let mut file = mono(&[0, 16_384]);
    let mut with_list = file.drain(..12).collect::<Vec<u8>>();
    with_list.extend_from_slice(b"LIST");
    with_list.extend_from_slice(&5u32.to_le_bytes());
    with_list.extend_from_slice(b"INFO\0");
    with_list.push(0);
    with_list.extend_from_slice(&file);

    assert_eq!(wav::decode(&with_list).unwrap().samples().len(), 2);
}
