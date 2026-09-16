use super::Running;

#[derive(Debug)]
pub struct Erasing {
    running: Running,
    program: String,
}

impl Erasing {
    pub(super) fn new(running: Running, program: String) -> Self {
        Self { running, program }
    }
}

impl Drop for Erasing {
    fn drop(&mut self) {
        self.running.erased(&self.program);
    }
}
