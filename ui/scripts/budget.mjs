import { gzipSync } from "node:zlib";
import { readFileSync } from "node:fs";
import { readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const DIST = fileURLToPath(new URL("../dist", import.meta.url));

const LIMITS = [
  ["/topic/", 30 * 1024],
  ["/graph/", 60 * 1024],
  ["/", 15 * 1024],
];

const OTHERS = 20 * 1024;

export function limit(route) {
  const found = LIMITS.find(([known]) => known === route);
  return found ? found[1] : OTHERS;
}

export async function pages(dist = DIST) {
  const found = [];
  for (const entry of await readdir(dist, { recursive: true, withFileTypes: true })) {
    if (entry.isFile() && entry.name.endsWith(".html")) {
      found.push(path.join(entry.parentPath, entry.name));
    }
  }
  return found.sort();
}

export function route(file, dist = DIST) {
  const relative = path.relative(dist, file).split(path.sep).slice(0, -1).join("/");
  return relative ? `/${relative}/` : "/";
}

export function scripts(html) {
  const inline = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)]
    .filter((match) => !/type=["'](application\/json|application\/ld\+json)["']/.test(match[0]))
    .map((match) => match[1]);
  const linked = [
    ...html.matchAll(/<script[^>]*\bsrc=["']([^"']+)["']/g),
    ...html.matchAll(/<link[^>]*\brel=["']modulepreload["'][^>]*\bhref=["']([^"']+)["']/g),
  ].map((match) => match[1]);
  return { inline, linked: [...new Set(linked)] };
}

function chunks(source) {
  return [...source.matchAll(/["'`](\/[^"'`\s]+\.m?js)["'`]/g)].map((match) => match[1]);
}

export function weight(file, dist = DIST) {
  const html = readFileSync(file, "utf8");
  const { inline, linked } = scripts(html);
  const seen = new Set();
  const source = [...inline];
  const queue = [...linked, ...chunks(html)];

  while (queue.length > 0) {
    const next = queue.shift();
    if (!next.startsWith("/") || seen.has(next)) continue;
    seen.add(next);
    const code = readFileSync(path.join(dist, next), "utf8");
    source.push(code);
    queue.push(...chunks(code));
  }

  const joined = source.join("\n");
  return { bytes: joined.trim() ? gzipSync(joined).length : 0, files: [...seen] };
}

export async function measure(dist = DIST) {
  return (await pages(dist)).map((file) => {
    const where = route(file, dist);
    const { bytes, files } = weight(file, dist);
    return { route: where, bytes, files, limit: limit(where), ok: bytes <= limit(where) };
  });
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const measured = await measure();
  for (const { route: where, bytes, limit: allowed, ok } of measured) {
    const mark = ok ? "ok" : "ПРЕВЫШЕНО";
    console.log(`${where.padEnd(12)} ${String(bytes).padStart(6)} / ${allowed} байт (gzip) — ${mark}`);
  }
  if (measured.some(({ ok }) => !ok)) {
    console.error("вес JS вышел за бюджет из docs/architecture.md");
    process.exit(1);
  }
}
