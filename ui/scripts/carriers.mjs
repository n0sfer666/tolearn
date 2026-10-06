import { readFileSync } from "node:fs";
import { readdir } from "node:fs/promises";
import path from "node:path";

const INTERACTIVE = ["a", "button", "summary"];
const MARKUP = /\.(?:tsx|astro)$/;
const ATTRIBUTE = /(?<=\s)(data-[\w-]+)/g;

function opening(source, from) {
  let depth = 0;

  for (let at = from; at < source.length; at += 1) {
    if (source[at] === "{") depth += 1;
    if (source[at] === "}") depth -= 1;
    if (source[at] === ">" && depth === 0) return source.slice(from, at);
  }

  return source.slice(from);
}

export function carried(source, tags = INTERACTIVE) {
  const found = new Set();
  const tag = new RegExp(`<(?:${tags.join("|")})(?=[\\s/>])`, "g");

  for (const { index } of source.matchAll(tag)) {
    for (const [name] of opening(source, index).matchAll(ATTRIBUTE)) found.add(name);
  }

  return found;
}

export async function carriers(dir, tags = INTERACTIVE) {
  const found = new Set();

  for (const entry of await readdir(dir, { recursive: true, withFileTypes: true })) {
    if (!entry.isFile() || !MARKUP.test(entry.name)) continue;
    const source = readFileSync(path.join(entry.parentPath, entry.name), "utf8");
    for (const name of carried(source, tags)) found.add(name);
  }

  return found;
}
