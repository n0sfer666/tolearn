use crate::yaml::{ParseError, Reader};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hours {
    pub min: u32,
    pub max: u32,
}

pub(crate) fn hours(node: &Reader<'_>) -> Result<Hours, ParseError> {
    let items = node.items()?;
    let [min, max] = items.as_slice() else {
        return Err(node.malformed("expected a list of exactly two numbers"));
    };
    Ok(Hours {
        min: min.number(1)?,
        max: max.number(1)?,
    })
}
