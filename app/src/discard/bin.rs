use std::path::Path;

use trash::TrashContext;

use super::Bin;

#[derive(Debug, Clone)]
pub struct Trash {
    context: TrashContext,
}

impl Trash {
    pub fn new() -> Self {
        let mut context = TrashContext::new();
        quietly(&mut context);
        Self { context }
    }
}

impl Default for Trash {
    fn default() -> Self {
        Self::new()
    }
}

impl Bin for Trash {
    fn discard(&self, path: &Path) -> Result<(), String> {
        self.context.delete(path).map_err(|error| error.to_string())
    }
}

#[cfg(target_os = "macos")]
fn quietly(context: &mut TrashContext) {
    use trash::macos::{DeleteMethod, TrashContextExtMacos};

    context.set_delete_method(DeleteMethod::NsFileManager);
}

#[cfg(not(target_os = "macos"))]
fn quietly(_context: &mut TrashContext) {}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use trash::macos::{DeleteMethod, TrashContextExtMacos};

    use super::Trash;

    #[test]
    fn корзина_не_просит_прав_на_управление_finder() {
        let bin = Trash::new();

        assert!(matches!(
            bin.context.delete_method(),
            DeleteMethod::NsFileManager
        ));
    }
}
