mod dates;
mod duplicates;
mod error;
mod failure;
mod reader;

pub use error::ParseError;
pub use failure::ParseFailure;
pub use reader::Reader;

use saphyr::{LoadableYamlNode, MarkedYaml};

const ONE_DOCUMENT: &str = "a bundle file holds exactly one YAML document";

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
