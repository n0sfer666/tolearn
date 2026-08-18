pub const HARVEST: &str = "window.__tolearn_prerendered ?? null";

pub fn watcher(settle: u64) -> String {
    format!(
        "(() => {{
  const harvest = () => {{
    window.__tolearn_prerendered = document.documentElement.outerHTML;
  }};
  const start = () => setTimeout(harvest, {settle});
  if (document.readyState === 'complete') {{
    start();
  }} else {{
    window.addEventListener('load', start);
  }}
}})();"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ожидание_тишины_попадает_в_скрипт() {
        assert!(watcher(700).contains("setTimeout(harvest, 700)"));
    }

    #[test]
    fn скрипт_ждёт_загрузку_если_она_ещё_идёт() {
        let script = watcher(500);
        assert!(script.contains("document.readyState === 'complete'"));
        assert!(script.contains("window.addEventListener('load', start)"));
    }
}
