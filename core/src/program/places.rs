use std::ops::Range;

use crate::yaml::{ParseError, Reader, read};

pub(crate) fn places(source: &str) -> Result<Vec<(Range<usize>, String)>, ParseError> {
    read(source, |node| {
        let mut places = vec![place(&node.field("uuid")?)?];
        places.extend(
            node.field("map")?
                .field("children")?
                .list(|row| place(&row.field("uuid")?))?,
        );
        Ok(places)
    })
}

fn place(node: &Reader<'_>) -> Result<(Range<usize>, String), ParseError> {
    Ok((node.chars(), node.uuid()?))
}
