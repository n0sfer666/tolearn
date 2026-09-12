use quick_xml::events::{BytesDecl, BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

const FOREIGN: [&[u8]; 5] = [b"script", b"foreignobject", b"iframe", b"embed", b"object"];

pub fn check(data: &[u8]) -> Result<(), String> {
    if let Some(byte) = data
        .iter()
        .find(|byte| **byte < 0x20 && !matches!(byte, b'\t' | b'\n' | b'\r'))
    {
        return Err(format!("it carries the control byte 0x{byte:02x}"));
    }
    let text = std::str::from_utf8(data).map_err(|_| "it is not UTF-8".to_owned())?;
    let mut reader = Reader::from_str(text);
    loop {
        match reader.read_event().map_err(broken)? {
            Event::Start(element) | Event::Empty(element) => tag(&element)?,
            Event::Decl(declaration) => encoding(&declaration)?,
            Event::DocType(doctype) if doctype.contains(&b'[') => {
                return Err("its DOCTYPE declares entities of its own".to_owned());
            }
            Event::PI(instruction) => {
                return Err(format!(
                    "it carries the instruction `<?{}`",
                    String::from_utf8_lossy(instruction.target())
                ));
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
    }
}

fn encoding(declaration: &BytesDecl<'_>) -> Result<(), String> {
    match declaration.encoding() {
        Some(Ok(name)) if !name.eq_ignore_ascii_case(b"utf-8") => Err(format!(
            "it declares the encoding `{}`",
            String::from_utf8_lossy(&name)
        )),
        Some(Err(error)) => Err(broken(error)),
        _ => Ok(()),
    }
}

fn tag(element: &BytesStart<'_>) -> Result<(), String> {
    let local = element.local_name();
    if let Some(name) = FOREIGN
        .iter()
        .find(|name| local.as_ref().eq_ignore_ascii_case(name))
    {
        return Err(format!("it holds a `<{}>`", String::from_utf8_lossy(name)));
    }
    for attribute in element.attributes() {
        let attribute = attribute.map_err(broken)?;
        let key = attribute.key.local_name();
        let key = key.as_ref();
        let name = String::from_utf8_lossy(attribute.key.as_ref());
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(broken)?;
        if key.len() > 2 && key[..2].eq_ignore_ascii_case(b"on") {
            return Err(format!("its `{name}` runs a script"));
        }
        if key.eq_ignore_ascii_case(b"href") && !value.trim_start().starts_with('#') {
            return Err(format!("its `{name}` points outside the picture"));
        }
        if key.eq_ignore_ascii_case(b"attributename") && animates_link(&value) {
            return Err(format!("its `{name}` animates a link"));
        }
    }
    Ok(())
}

fn animates_link(value: &str) -> bool {
    value
        .rsplit(':')
        .next()
        .is_some_and(|local| local.trim().eq_ignore_ascii_case("href"))
}

fn broken(error: impl std::fmt::Display) -> String {
    format!("it is not XML: {error}")
}
