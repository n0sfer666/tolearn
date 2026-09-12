#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

use tolearn_core::block::{self, Block, Kind};
use tolearn_generate::diagram::{Diagram, Painter, draw};
use tolearn_generate::sources::Illustration;

const SOURCE: &str = "flowchart LR\n  A[Лента] --> B[Головка]";
const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><style>#d{fill:red}</style><use href="#a"/><text>Лента</text></svg>"##;

struct Canvas(Result<&'static str, &'static str>);

impl Painter for Canvas {
    fn paint(&self, mermaid: &str) -> Result<String, String> {
        assert_eq!(mermaid, SOURCE);
        self.0.map(str::to_owned).map_err(str::to_owned)
    }
}

fn drawn(diagram: Diagram) -> Illustration {
    match diagram {
        Diagram::Drawn(drawn) => drawn,
        Diagram::Source { reason, .. } => panic!("схема не нарисована: {reason}"),
    }
}

fn degraded(diagram: Diagram) -> (Block, String) {
    match diagram {
        Diagram::Source { block, reason } => (block, reason),
        Diagram::Drawn(drawn) => panic!("схема нарисована: {drawn:?}"),
    }
}

#[test]
fn a_drawn_diagram_becomes_a_diagram_block_with_its_svg_asset() {
    let drawn = drawn(draw(&Canvas(Ok(SVG)), SOURCE));

    assert_eq!(drawn.bytes, SVG.as_bytes());
    assert!(drawn.file.ends_with(".svg"), "{}", drawn.file);
    assert_eq!(drawn.file.len(), 16 + ".svg".len());
    let block = drawn.block;
    assert_eq!(block.kind, Kind::Diagram);
    assert_eq!(block.id, block::id(SOURCE));
    assert_eq!(block.text, SOURCE);
    assert_eq!(block.asset, Some(format!("assets/{}", drawn.file)));
    assert_eq!(block.lang, None);
    assert_eq!(block.license, None);
}

#[test]
fn broken_mermaid_degrades_to_a_code_block_with_its_source() {
    let (block, reason) = degraded(draw(&Canvas(Err("Parse error on line 2")), SOURCE));

    assert_eq!(block.kind, Kind::Code);
    assert_eq!(block.lang.as_deref(), Some("mermaid"));
    assert_eq!(block.text, SOURCE);
    assert_eq!(block.id, block::id(SOURCE));
    assert_eq!(block.asset, None);
    assert!(reason.contains("Parse error on line 2"), "{reason}");
}

#[test]
fn an_svg_that_fails_the_import_check_degrades_to_code() {
    for svg in [
        r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)"></svg>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg"><a href="https://example.com"><text>x</text></a></svg>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject><div>x</div></foreignObject></svg>"#,
    ] {
        let (block, reason) = degraded(draw(&Canvas(Ok(svg)), SOURCE));

        assert_eq!(block.kind, Kind::Code, "{svg}");
        assert!(!reason.is_empty(), "{svg}");
    }
}

#[test]
fn an_answer_that_is_not_svg_degrades_to_code() {
    for answer in ["", "   ", "<html><body>x</body></html>"] {
        let (block, _) = degraded(draw(&Canvas(Ok(answer)), SOURCE));

        assert_eq!(block.kind, Kind::Code, "{answer:?}");
    }
}
