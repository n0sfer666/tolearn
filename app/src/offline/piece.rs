use tolearn_core::scan::Scan;
use tolearn_core::topic::Material;

#[derive(Debug, Clone)]
pub struct Piece {
    pub topic: String,
    pub material: Material,
}

pub fn every(scan: &Scan) -> Vec<Piece> {
    let mut taken: Vec<String> = Vec::new();
    let mut pieces = Vec::new();
    for topic in &scan.topics {
        for material in &topic.materials {
            if taken.contains(&material.url) {
                continue;
            }
            taken.push(material.url.clone());
            pieces.push(Piece {
                topic: topic.title.clone(),
                material: material.clone(),
            });
        }
    }
    pieces
}

pub fn of(scan: &Scan, topic: &str) -> Option<Vec<Piece>> {
    scan.topics
        .iter()
        .find(|document| document.id == topic)
        .map(|document| {
            document
                .materials
                .iter()
                .map(|material| Piece {
                    topic: document.title.clone(),
                    material: material.clone(),
                })
                .collect()
        })
}
