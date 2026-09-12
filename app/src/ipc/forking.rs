use tolearn_generate::fork::{After, Fork};

use super::forked::{ForkOut, VariantView};
use super::types::Span;

pub fn after<'a>(program: &'a str, node: &'a str, stage: &'a str) -> After<'a> {
    After {
        program,
        node: if node.is_empty() { program } else { node },
        stage,
    }
}

pub fn view(fork: &Fork) -> ForkOut {
    ForkOut {
        variants: fork
            .variants
            .iter()
            .map(|variant| VariantView {
                id: variant.row.id.clone(),
                title: variant.row.title.clone(),
                hours: Span {
                    min: variant.row.hours.min,
                    max: variant.row.hours.max,
                },
                why: variant.why.clone(),
                recommended: variant.recommended,
            })
            .collect(),
    }
}
