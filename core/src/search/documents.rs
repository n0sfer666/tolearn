use super::collect::Wanted;
use super::types::{Document, Kind, Source};
use crate::block;
use crate::library::{Library, LibraryError, Refusal};
use crate::program::{LoadError, Tree};

pub fn source(library: &Library, wanted: Wanted) -> Option<Source> {
    let documents = match library.open(&wanted.program) {
        Ok(tree) => {
            let mut documents = Vec::new();
            node(&tree, &mut documents);
            documents
        }
        Err(error) if unreadable(&error) => return None,
        Err(_) => Vec::new(),
    };
    Some(Source {
        program: wanted.program,
        files: wanted.files,
        documents,
    })
}

fn unreadable(error: &LibraryError) -> bool {
    matches!(
        error,
        LibraryError::Unreadable { .. }
            | LibraryError::Absent { .. }
            | LibraryError::Refused(Refusal::Unloadable(LoadError::Unreadable { .. }))
    )
}

fn node(tree: &Tree, out: &mut Vec<Document>) {
    let program = &tree.program;
    for row in &program.map.stages {
        let Some(stage) = tree.stages.get(&row.id) else {
            continue;
        };
        let document = |kind, block: &str, text: &str| Document {
            kind,
            node: program.uuid.clone(),
            node_title: program.title.clone(),
            stage: stage.id.clone(),
            title: stage.title.clone(),
            block: block.to_owned(),
            text: text.to_owned(),
        };
        out.push(document(Kind::Stage, "", ""));
        out.extend(
            stage
                .every_block()
                .filter(|block| block.kind != block::Kind::Diagram)
                .map(|block| document(Kind::Block, &block.id, &block.text)),
        );
    }
    for row in &program.map.children {
        if let Some(child) = tree.children.get(&row.uuid) {
            node(child, out);
        }
    }
}
