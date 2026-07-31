use crate::digest::digest;

const OPEN: &[u8] = b"<body";
const CLOSE: &[u8] = b"</body";
const REMARK: &[u8] = b"<!--";
const REMARK_END: &[u8] = b"-->";
const SCRIPT: &[u8] = b"<script";
const SCRIPT_END: &[u8] = b"</script";

pub fn body_hash(bytes: &[u8]) -> String {
    digest(&plain(inside(bytes)))
}

fn inside(bytes: &[u8]) -> &[u8] {
    let Some(open) = tag(bytes, OPEN) else {
        return bytes;
    };
    let Some(head) = bytes[open..].iter().position(|byte| *byte == b'>') else {
        return bytes;
    };
    let from = open + head + 1;
    match tag(&bytes[from..], CLOSE) {
        Some(close) => &bytes[from..from + close],
        None => &bytes[from..],
    }
}

fn tag(bytes: &[u8], needle: &[u8]) -> Option<usize> {
    let mut from = 0;
    while from < bytes.len() {
        if starts(&bytes[from..], REMARK) {
            from = past(bytes, from + REMARK.len(), REMARK_END);
            continue;
        }
        if starts(&bytes[from..], SCRIPT) {
            from = past(bytes, from + SCRIPT.len(), SCRIPT_END);
            continue;
        }
        if starts(&bytes[from..], needle) && edged(bytes.get(from + needle.len())) {
            return Some(from);
        }
        from += 1;
    }
    None
}

fn past(bytes: &[u8], from: usize, needle: &[u8]) -> usize {
    let mut at = from;
    while at < bytes.len() {
        if starts(&bytes[at..], needle) {
            return at + needle.len();
        }
        at += 1;
    }
    bytes.len()
}

fn starts(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.len() >= needle.len() && bytes[..needle.len()].eq_ignore_ascii_case(needle)
}

fn edged(byte: Option<&u8>) -> bool {
    byte.is_some_and(|byte| *byte == b'>' || *byte == b'/' || byte.is_ascii_whitespace())
}

fn plain(bytes: &[u8]) -> Vec<u8> {
    let mut plain = Vec::with_capacity(bytes.len());
    let mut spaced = true;
    for byte in bytes {
        if byte.is_ascii_whitespace() {
            if !spaced {
                plain.push(b' ');
            }
            spaced = true;
            continue;
        }
        plain.push(*byte);
        spaced = false;
    }
    while plain.last() == Some(&b' ') {
        plain.pop();
    }
    plain
}
