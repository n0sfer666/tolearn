mod blocks;
mod error;
mod escape;
mod index;
mod lines;
mod place;
mod prose;
mod stage;
mod within;
mod words;
mod write;

use crate::program::Tree;

pub use error::ExportError;
pub use place::{claimed, inside};
pub use within::within;
pub use write::write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    pub pages: Vec<Page>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub path: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub from: String,
    pub to: String,
}

pub fn render(tree: &Tree) -> Export {
    let mut export = Export {
        pages: Vec::new(),
        assets: Vec::new(),
    };
    walk(tree, "", "", None, &mut export);
    export
}

fn walk(tree: &Tree, from: &str, to: &str, parent: Option<&str>, export: &mut Export) {
    let program = &tree.program;
    let words = words::of(&program.generation.locale);
    export.pages.push(Page {
        path: format!("{to}index.md"),
        text: index::page(tree, parent, words),
    });
    for (place, row) in program.map.stages.iter().enumerate() {
        if let Some(found) = tree.stages.get(&row.id) {
            export.pages.push(Page {
                path: format!("{to}{}", file(place, &row.id)),
                text: stage::page(found, &program.title, words),
            });
        }
    }
    export.assets.extend(tree.assets.iter().map(|asset| Asset {
        from: format!("{from}{asset}"),
        to: format!("{to}{asset}"),
    }));
    for (place, row) in program.map.children.iter().enumerate() {
        if let Some(child) = tree.children.get(&row.uuid) {
            walk(
                child,
                &format!("{from}children/{}/", row.uuid),
                &format!("{to}{}/", folder(place, &child.program.slug)),
                Some(&program.title),
                export,
            );
        }
    }
}

fn file(place: usize, id: &str) -> String {
    format!("{:02}-{id}.md", place + 1)
}

fn folder(place: usize, slug: &str) -> String {
    format!("{:02}-{slug}", place + 1)
}
