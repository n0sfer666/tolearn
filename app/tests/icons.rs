#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "manifest gate: a panic here is the report"
)]

use std::io::Read;
use std::path::{Path, PathBuf};

fn root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name)
}

fn corner(icon: &str) -> [u8; 4] {
    let path = root(icon);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(reason) => panic!("{}: {reason}", path.display()),
    };
    let header = &bytes[16..29];
    assert_eq!(header[8], 8, "{icon}: не восемь бит на канал");
    assert_eq!(header[9], 6, "{icon}: не RGBA — прозрачности негде взяться");
    assert_eq!(
        header[12], 0,
        "{icon}: чересстрочный PNG, первый пиксель не в первой строке"
    );

    let mut pixels = Vec::new();
    let mut at = 8;
    let mut stream = Vec::new();
    while at + 8 <= bytes.len() {
        let size = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        if &bytes[at + 4..at + 8] == b"IDAT" {
            stream.extend_from_slice(&bytes[at + 8..at + 8 + size]);
        }
        at += 12 + size;
    }
    flate2::read::ZlibDecoder::new(stream.as_slice())
        .take(5)
        .read_to_end(&mut pixels)
        .unwrap();

    [pixels[1], pixels[2], pixels[3], pixels[4]]
}

#[test]
fn растры_не_несут_белой_подложки() {
    for icon in [
        "app/icons/32x32.png",
        "app/icons/128x128.png",
        "app/icons/128x128@2x.png",
        "app/icons/icon.png",
    ] {
        let pixel = corner(icon);

        assert_eq!(
            pixel[3], 0,
            "{icon}: угол непрозрачен ({pixel:?}) — вокруг squircle осталась подложка"
        );
    }
}
