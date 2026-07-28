use tolearn_core::yaml::{ParseError, read};

use crate::types::{FLAVOR, Provider};

pub fn provider(source: &str) -> Result<Provider, ParseError> {
    read(source, |node| {
        Ok(Provider {
            enabled: node.field("enabled")?.flag()?,
            flavor: node.field("flavor")?.choice("flavor", &FLAVOR)?,
            endpoint: node.field("endpoint")?.text()?,
            model: node.field("model")?.any_text()?,
        })
    })
}
