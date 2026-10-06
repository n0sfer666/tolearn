use std::path::Path;

use serde::{Deserialize, Serialize};
use tolearn_core::block::{Block, Kind};
use tolearn_core::program::StageRow;

use crate::error::GenerateError;
use crate::sources::{Cache, Illustration, Verified, excerpt};
use crate::stage::{Chaptered, Dropped, Gathered, Visited};

const GATHERED: &str = "gathered";
const PICTURES: &str = "pictures";

#[derive(Serialize, Deserialize)]
struct Kept {
    row: String,
    books: Vec<Chaptered>,
    pages: Vec<Visited>,
    images: Vec<Picture>,
    dropped: Vec<Dropped>,
}

#[derive(Serialize, Deserialize)]
struct Picture {
    file: String,
    id: String,
    caption: String,
    asset: Option<String>,
    attribution: Option<String>,
    license: Option<String>,
    source: Option<String>,
}

pub(crate) fn save(
    data: &Path,
    program: &str,
    row: &StageRow,
    gathered: &Gathered,
) -> Result<(), GenerateError> {
    let cache = Cache::at(data, program);
    for image in &gathered.images {
        cache.save_bytes(PICTURES, &image.file, &image.bytes)?;
    }
    let kept = Kept {
        row: marked(row),
        books: gathered.books.clone(),
        pages: gathered.pages.iter().map(trimmed).collect(),
        images: gathered.images.iter().map(kept).collect(),
        dropped: gathered.dropped.clone(),
    };
    cache.save(GATHERED, &row.id, &kept)
}

pub(crate) fn load(data: &Path, program: &str, row: &StageRow) -> Option<Gathered> {
    let cache = Cache::at(data, program);
    let kept: Kept = cache.load(GATHERED, &row.id).ok().flatten()?;
    if kept.row != marked(row) {
        return None;
    }
    let images = kept
        .images
        .into_iter()
        .map(|picture| drawn(&cache, picture))
        .collect::<Option<Vec<_>>>()?;
    Some(Gathered {
        books: kept.books,
        pages: kept.pages,
        images,
        dropped: kept.dropped,
    })
}

fn marked(row: &StageRow) -> String {
    format!(
        "{}\n{}\n{}–{}",
        row.id, row.title, row.hours.min, row.hours.max
    )
}

fn trimmed(visited: &Visited) -> Visited {
    Visited {
        page: Verified {
            text: excerpt(&visited.page.text).to_owned(),
            ..visited.page.clone()
        },
        checked_at: visited.checked_at,
    }
}

fn kept(image: &Illustration) -> Picture {
    let block = &image.block;
    Picture {
        file: image.file.clone(),
        id: block.id.clone(),
        caption: block.text.clone(),
        asset: block.asset.clone(),
        attribution: block.attribution.clone(),
        license: block.license.clone(),
        source: block.source.clone(),
    }
}

fn drawn(cache: &Cache, picture: Picture) -> Option<Illustration> {
    let bytes = cache.load_bytes(PICTURES, &picture.file).ok().flatten()?;
    Some(Illustration {
        file: picture.file,
        bytes,
        block: Block {
            id: picture.id,
            kind: Kind::Image,
            text: picture.caption,
            lang: None,
            asset: picture.asset,
            attribution: picture.attribution,
            license: picture.license,
            source: picture.source,
        },
    })
}
