use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const GLANCE: Duration = Duration::from_millis(20);

pub(super) fn size(at: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![at.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(facts) = entry.metadata() {
                total += facts.len();
            }
        }
    }
    total
}

pub(super) fn watch(at: &Path, limit: u64, stop: &AtomicBool, done: &AtomicBool) {
    while !done.load(Ordering::Relaxed) {
        if size(at) > limit {
            stop.store(true, Ordering::Relaxed);
            return;
        }
        std::thread::sleep(GLANCE);
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "offline gate: a panic here is the report"
    )]

    use super::*;

    fn plot(name: &str) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("tolearn-weigh-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("глубже")).unwrap();
        path
    }

    #[test]
    fn объём_считается_по_всем_подкаталогам() {
        let at = plot("size");
        std::fs::write(at.join("верх.bin"), vec![b'x'; 300]).unwrap();
        std::fs::write(at.join("глубже/низ.bin"), vec![b'x'; 700]).unwrap();

        assert_eq!(size(&at), 1000);

        let _ = std::fs::remove_dir_all(&at);
    }

    #[test]
    fn перебор_объёма_поднимает_флаг_остановки() {
        let at = plot("over");
        std::fs::write(at.join("глубже/низ.bin"), vec![b'x'; 1000]).unwrap();
        let stop = AtomicBool::new(false);
        let done = AtomicBool::new(false);

        watch(&at, 999, &stop, &done);

        assert!(stop.load(Ordering::Relaxed), "сторож не прервал клон");

        let _ = std::fs::remove_dir_all(&at);
    }

    #[test]
    fn в_рамках_объёма_сторож_молчит() {
        let at = plot("under");
        std::fs::write(at.join("глубже/низ.bin"), vec![b'x'; 1000]).unwrap();
        let stop = AtomicBool::new(false);
        let done = AtomicBool::new(false);

        std::thread::scope(|scope| {
            let watchman = scope.spawn(|| watch(&at, 1000, &stop, &done));
            std::thread::sleep(GLANCE * 5);
            done.store(true, Ordering::Relaxed);
            let _ = watchman.join();
        });

        assert!(
            !stop.load(Ordering::Relaxed),
            "сторож прервал разрешённый клон"
        );

        let _ = std::fs::remove_dir_all(&at);
    }
}
