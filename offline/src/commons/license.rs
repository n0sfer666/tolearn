pub fn free(code: &str) -> bool {
    let code = code.to_ascii_lowercase();
    code == "cc0"
        || code == "pd"
        || code.starts_with("pd-")
        || versioned(&code, "cc-by-")
        || versioned(&code, "cc-by-sa-")
}

fn versioned(code: &str, family: &str) -> bool {
    code.strip_prefix(family)
        .is_some_and(|rest| rest.starts_with(|symbol: char| symbol.is_ascii_digit()))
}
