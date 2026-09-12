mod painter;

pub use painter::Painter;

use tolearn_core::block::{self, Block, Kind};
use tolearn_core::package::svg;

use crate::sources::{Illustration, file_name};

const LANG: &str = "mermaid";
const EXTENSION: &str = "svg";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagram {
    Drawn(Illustration),
    Source { block: Block, reason: String },
}

pub fn draw(painter: &dyn Painter, mermaid: &str) -> Diagram {
    match painter.paint(mermaid).and_then(checked) {
        Ok(svg) => Diagram::Drawn(drawn(mermaid, svg.into_bytes())),
        Err(reason) => Diagram::Source {
            block: source(mermaid),
            reason,
        },
    }
}

fn checked(svg: String) -> Result<String, String> {
    if !svg.trim_start().starts_with("<svg") {
        return Err("окно схемы вернуло не SVG".to_owned());
    }
    svg::check(svg.as_bytes())
        .map_err(|reason| format!("SVG схемы не прошёл проверку импорта: {reason}"))?;
    Ok(svg)
}

fn drawn(mermaid: &str, bytes: Vec<u8>) -> Illustration {
    let file = file_name(&bytes, EXTENSION);
    let block = Block {
        id: block::id(mermaid),
        kind: Kind::Diagram,
        text: mermaid.to_owned(),
        lang: None,
        asset: Some(format!("assets/{file}")),
        license: None,
        attribution: None,
        source: None,
    };
    Illustration { file, bytes, block }
}

fn source(mermaid: &str) -> Block {
    Block {
        id: block::id(mermaid),
        kind: Kind::Code,
        text: mermaid.to_owned(),
        lang: Some(LANG.to_owned()),
        asset: None,
        license: None,
        attribution: None,
        source: None,
    }
}
