const COMMENT = /\/\*[\s\S]*?\*\//g;

export function blocks(css) {
  const source = css.replace(COMMENT, "");
  const found = [];
  const stack = [];
  let start = 0;

  for (let at = 0; at < source.length; at += 1) {
    if (source[at] === "{") {
      stack.push({ prelude: source.slice(start, at).trim(), body: at + 1 });
      start = at + 1;
      continue;
    }
    if (source[at] !== "}") continue;

    const open = stack.pop();
    if (!open) continue;
    const body = source.slice(open.body, at);
    if (!open.prelude.startsWith("@")) {
      const outer = stack.find(({ prelude }) => prelude.startsWith("@media"));
      found.push({
        selector: open.prelude,
        media: outer ? outer.prelude.slice("@media".length).trim() : null,
        body,
      });
    }
    start = at + 1;
  }

  return found;
}

export function declarations(body) {
  return body
    .split(";")
    .map((piece) => piece.trim())
    .filter((piece) => piece.includes(":") && !piece.includes("{"))
    .map((piece) => {
      const at = piece.indexOf(":");
      return { property: piece.slice(0, at).trim(), value: piece.slice(at + 1).trim() };
    });
}
