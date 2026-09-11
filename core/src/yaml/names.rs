use super::error::ParseError;
use super::reader::Reader;

impl Reader<'_> {
    pub fn slug(&self) -> Result<String, ParseError> {
        let text = self.text()?;
        if !is_slug(&text) {
            return Err(self.unknown(format!(
                "`{text}` is no slug of lowercase letters, digits and single hyphens"
            )));
        }
        Ok(text)
    }

    pub fn uuid(&self) -> Result<String, ParseError> {
        let text = self.text()?;
        if !is_uuid(&text) {
            return Err(self.unknown(format!("`{text}` is no UUID in lowercase hex")));
        }
        Ok(text)
    }
}

fn is_slug(text: &str) -> bool {
    text.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

pub(crate) fn is_uuid(text: &str) -> bool {
    let groups: Vec<&str> = text.split('-').collect();
    groups.iter().map(|group| group.len()).eq([8, 4, 4, 4, 12])
        && groups.iter().all(|group| {
            group
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        })
}
