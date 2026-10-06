mod duplicates;
mod emit;
mod error;
mod failure;
mod names;
mod reader;

pub use emit::dump;
pub use error::ParseError;
pub use failure::ParseFailure;
pub use names::is_slug;
pub use reader::Reader;

pub(crate) use emit::{flag, list, map, number, text};
pub(crate) use names::is_uuid;

use saphyr::{LoadableYamlNode, MarkedYaml};

const ONE_DOCUMENT: &str = "a file holds exactly one YAML document";

pub fn read<T>(
    source: &str,
    parse: impl FnOnce(&Reader<'_>) -> Result<T, ParseError>,
) -> Result<T, ParseError> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    duplicates::check(source)?;
    let documents = MarkedYaml::load_from_str(source).map_err(ParseError::from_scan)?;
    match documents.as_slice() {
        [root] => parse(&Reader::root(root)),
        [] => Err(ParseError::at_start(
            ParseFailure::DocumentCount,
            format!("{ONE_DOCUMENT}, this one holds none"),
        )),
        [_, second, ..] => Err(ParseError::at(
            ParseFailure::DocumentCount,
            second.span.start,
            "",
            format!("{ONE_DOCUMENT}, this one holds more"),
        )),
    }
}
