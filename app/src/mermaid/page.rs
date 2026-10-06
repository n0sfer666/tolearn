pub(super) const MERMAID: &[u8] = include_bytes!("../../vendor/mermaid/mermaid.tiny.js");

pub(super) const INDEX: &str = "<!doctype html><html><head><meta charset=\"utf-8\"></head>\
<body><script src=\"mermaid.tiny.js\"></script><script src=\"draw.js\"></script></body></html>";

pub(super) const DRAW: &str = r#"(async () => {
  try {
    const source = decodeURIComponent(location.hash.slice(1));
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      htmlLabels: false,
      flowchart: { htmlLabels: false },
      theme: "neutral",
    });
    const { svg } = await mermaid.render("diagram", source);
    const holder = document.createElement("div");
    holder.innerHTML = svg;
    window.__tolearn = { svg: new XMLSerializer().serializeToString(holder.firstElementChild) };
  } catch (error) {
    window.__tolearn = { error: String(error && error.message ? error.message : error) };
  }
})();
"#;
