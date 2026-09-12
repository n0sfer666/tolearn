use tolearn_generate::ledger::{Kind, Tally};
use tolearn_offline::book::Wanted;

use crate::ledger::{ISBN, QUERY, shape};
use crate::web::{ARTICLE, Bench};

const MISSING: &str = "https://a.test/missing";

#[test]
fn a_source_answered_from_the_cache_is_not_counted_again() {
    let mut bench = Bench::new("ledger-cache");
    let tally = Tally::default();
    let wanted = Wanted {
        isbn: Some(ISBN.to_owned()),
        title: "Game Sound".to_owned(),
        author: "Karen Collins".to_owned(),
    };
    let mut sources = bench.sources().counted(&tally);
    for _ in 0..2 {
        sources.page(ARTICLE).unwrap();
        sources.book(&wanted).unwrap();
        sources.image(QUERY, "Импульсная волна");
        sources.page(MISSING).unwrap();
    }

    let records = tally.records();
    assert_eq!(
        shape(&records),
        [
            ("sources", Kind::Page, Some(ARTICLE)),
            ("sources", Kind::Book, Some(ISBN)),
            ("sources", Kind::Image, Some(QUERY)),
            ("sources", Kind::Page, Some(MISSING)),
            ("sources", Kind::Image, Some(QUERY)),
            ("sources", Kind::Page, Some(MISSING)),
        ]
    );
    let refused: Vec<bool> = records.iter().map(|record| record.ok).collect();
    assert_eq!(refused, [true, true, true, false, true, false]);
}
