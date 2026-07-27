import { readFileSync } from "node:fs";
import { readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { blocks, declarations } from "./css.mjs";
import { ratio, themes } from "./contrast.mjs";

export const SOURCE = fileURLToPath(new URL("../../docs/design/tokens.css", import.meta.url));

const SRC = fileURLToPath(new URL("../src", import.meta.url));

const FAMILIES = [
  "color",
  "spacing",
  "font-size",
  "font-family",
  "line-height",
  "radius",
  "z",
  "breakpoint",
];

const SURFACES = ["--color-bg", "--color-bg-subtle", "--color-bg-raised"];
const TEXT = [
  "--color-fg",
  "--color-fg-muted",
  "--color-neutral",
  "--color-accent",
  "--color-warning",
  "--color-critical",
];
const OUTLINES = { "--color-border-strong": 3, "--color-focus": 3 };

const HEX = /#[0-9a-fA-F]{3,8}\b/;
const PX = /\b\d+(?:\.\d+)?px\b/g;
const BREAKPOINTS = ["640px", "1024px"];

export function read() {
  return readFileSync(SOURCE, "utf8");
}

export function tokens(css) {
  const found = new Map();
  for (const { body } of blocks(css)) {
    for (const { property, value } of declarations(body)) {
      if (property.startsWith("--")) found.set(property, value);
    }
  }
  return found;
}

export async function styles() {
  const found = [];
  for (const entry of await readdir(SRC, { recursive: true, withFileTypes: true })) {
    if (entry.isFile() && /\.(css|astro)$/.test(entry.name)) {
      found.push(path.join(entry.parentPath, entry.name));
    }
  }
  return found.sort();
}

function family(name) {
  return FAMILIES.filter((known) => name.startsWith(`${known}-`) && name.length > known.length + 1).sort(
    (one, other) => other.length - one.length,
  )[0];
}

export function naming(names) {
  const problems = [];
  const sizes = new Map();
  const singles = new Map();

  for (const token of names) {
    const name = token.replace(/^--/, "");
    const owner = family(name);
    if (owner) {
      sizes.set(owner, (sizes.get(owner) ?? 0) + 1);
      continue;
    }
    if (!/^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(name)) {
      problems.push({ token, message: `${token}: имя вне kebab-case или без префикса семейства` });
      continue;
    }
    const head = name.split("-")[0];
    singles.set(head, [...(singles.get(head) ?? []), token]);
  }

  for (const [owner, size] of sizes) {
    if (size < 2) {
      problems.push({
        token: `--${owner}-*`,
        message: `--${owner}-*: семейство из одного члена — назови токен одним словом по смыслу`,
      });
    }
  }
  for (const [head, members] of singles) {
    if (members.length > 1) {
      problems.push({
        token: members[0],
        message: `${members.join(", ")}: общий сегмент «${head}» — заведи семейство --${head}-*`,
      });
    }
  }

  return problems;
}

function style(source) {
  if (!source.includes("<style")) return source;
  return [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((match) => match[1]).join("\n");
}

export function values(source, file) {
  const problems = [];

  for (const { selector, media, body } of blocks(style(source))) {
    const found = declarations(body);
    const where = `${file} → ${selector}`;

    for (const { property, value } of found) {
      if (HEX.test(value)) {
        problems.push({ file, message: `hex «${value}» в ${where} — цвет берётся из --color-*` });
      }
      for (const size of value.match(PX) ?? []) {
        if (property.startsWith("--")) continue;
        problems.push({ file, message: `px «${size}» в ${where} — размер берётся из токена` });
      }
      if (property === "outline" && /^(none|0)$/.test(value)) {
        const replaced = found.some(
          ({ property: other, value: instead }) =>
            other === "box-shadow" || (other.startsWith("outline") && other !== "outline" && instead !== "none"),
        );
        if (!replaced) problems.push({ file, message: `outline: none без замены в ${where}` });
      }
    }

    for (const size of media?.match(PX) ?? []) {
      if (!BREAKPOINTS.includes(size)) {
        problems.push({
          file,
          message: `брейкпойнт «${size}» в ${file} → @media ${media} — объявлены только ${BREAKPOINTS.join(", ")}`,
        });
      }
    }
  }

  return problems;
}

export function contrasts(map) {
  const problems = [];
  const pairs = [...TEXT.map((token) => [token, 4.5]), ...Object.entries(OUTLINES)];

  for (const theme of ["light", "dark"]) {
    for (const surface of SURFACES) {
      const behind = themes(map.get(surface))[theme];
      for (const [token, need] of pairs) {
        const front = themes(map.get(token))[theme];
        const measured = ratio(front, behind);
        if (measured + 0.005 < need) {
          problems.push({
            token,
            theme,
            surface,
            message: `${token} на ${surface} (${theme}): ${measured.toFixed(2)}:1 против ${need}:1`,
          });
        }
      }
    }
  }

  return problems;
}

export async function lint() {
  const map = tokens(read());
  const problems = [...naming([...map.keys()]), ...contrasts(map)];
  for (const file of await styles()) {
    problems.push(...values(readFileSync(file, "utf8"), path.relative(path.join(SRC, ".."), file)));
  }
  return problems;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const problems = await lint();
  for (const { message } of problems) console.error(message);
  console.log(problems.length === 0 ? "токены в порядке" : `нарушений: ${problems.length}`);
  if (problems.length > 0) process.exit(1);
}
