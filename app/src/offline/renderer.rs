use std::sync::OnceLock;

use tolearn_offline::page::{AsFetched, Prerenderer};

static INSTALLED: OnceLock<Box<dyn Prerenderer + Send + Sync>> = OnceLock::new();
static PLAIN: AsFetched = AsFetched;

pub fn install(renderer: Box<dyn Prerenderer + Send + Sync>) {
    let _ = INSTALLED.set(renderer);
}

pub fn current() -> &'static (dyn Prerenderer + Send + Sync) {
    match INSTALLED.get() {
        Some(renderer) => renderer.as_ref(),
        None => &PLAIN,
    }
}
