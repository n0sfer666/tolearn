pub(super) fn percent(line: &str) -> Option<f32> {
    let rest = line.strip_prefix("[download]")?.trim_start();
    let (number, _) = rest.split_once('%')?;
    number.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn доля_вынимается_из_строки_прогресса() {
        assert_eq!(
            percent("[download]  47.5% of 10.00MiB at 1.00MiB/s ETA 00:05"),
            Some(47.5)
        );
        assert_eq!(percent("[download] 100% of 10.00MiB in 00:10"), Some(100.0));
    }

    #[test]
    fn чужие_строки_не_считаются_прогрессом() {
        assert_eq!(percent("[youtube] Extracting URL"), None);
        assert_eq!(percent("[download] Destination: video.mp4"), None);
        assert_eq!(percent("ERROR: Video unavailable"), None);
        assert_eq!(percent("[download] 100 of 10.00MiB"), None);
    }
}
