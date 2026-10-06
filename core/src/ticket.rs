use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) fn ticket() -> String {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
