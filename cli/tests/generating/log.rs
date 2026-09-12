use std::io::{self, Write};
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Debug, Clone, Default)]
pub struct Log(Arc<Mutex<Vec<u8>>>);

impl Log {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap_or_else(PoisonError::into_inner)).into_owned()
    }

    pub fn clear(&self) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    pub fn names(&self) -> Vec<String> {
        self.text()
            .lines()
            .map(|line| line.split(" · ").next().unwrap_or_default().to_owned())
            .collect()
    }

    pub fn line(&self, name: &str) -> String {
        let text = self.text();
        text.lines()
            .find(|line| line.split(" · ").next() == Some(name))
            .unwrap_or_else(|| panic!("нет строки «{name}»: {text}"))
            .to_owned()
    }
}

impl Write for Log {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
