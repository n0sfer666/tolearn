use tolearn_generate::sources::{PAGE_CHARS, excerpt};

#[test]
fn text_for_the_model_is_cut_to_the_page_limit() {
    let long = "я".repeat(PAGE_CHARS + 10);

    assert_eq!(PAGE_CHARS, 8_000);
    assert_eq!(excerpt(&long).chars().count(), PAGE_CHARS);
    assert!(long.starts_with(excerpt(&long)));
    assert_eq!(excerpt("коротко"), "коротко");
}
