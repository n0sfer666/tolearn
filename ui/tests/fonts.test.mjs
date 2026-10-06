import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const UI = fileURLToPath(new URL("..", import.meta.url));
const FONTS = path.join(UI, "src/fonts");
const FACES = path.join(UI, "src/styles/fonts.css");
const RECORD = path.join(UI, "..", "THIRD-PARTY.md");

function shipped() {
  return readdirSync(FONTS)
    .filter((name) => name.endsWith(".woff2"))
    .sort();
}

function faces() {
  return [...readFileSync(FACES, "utf8").matchAll(/@font-face\s*{([^}]*)}/g)].map(([, body]) => ({
    family: body.match(/font-family:\s*"([^"]+)"/)?.[1],
    style: body.match(/font-style:\s*(\w+)/)?.[1],
    file: body.match(/url\("\.\.\/fonts\/([^"]+)"\)/)?.[1],
    range: body.match(/unicode-range:\s*([^;]+)/)?.[1] ?? "",
  }));
}

function script(range) {
  if (range.includes("U+0400-045F")) return "cyrillic";
  if (range.includes("U+0000-00FF")) return "latin";
  return "other";
}

test("у каждого файла шрифта есть строка в THIRD-PARTY.md с его sha256", () => {
  const rows = readFileSync(RECORD, "utf8").matchAll(/\| `([a-z-]+\.woff2)` \| `([0-9a-f]{64})` \|/g);
  const recorded = Object.fromEntries([...rows].map(([, name, sum]) => [name, sum]));
  const measured = Object.fromEntries(
    shipped().map((name) => [name, createHash("sha256").update(readFileSync(path.join(FONTS, name))).digest("hex")]),
  );

  assert.deepEqual(measured, recorded);
});

test("обе гарнитуры покрывают латиницу и кириллицу, у Literata есть курсив", () => {
  const covered = faces().map(({ family, style, range }) => `${family} ${style} ${script(range)}`);

  assert.deepEqual(covered.sort(), [
    "Golos Text normal cyrillic",
    "Golos Text normal latin",
    "Literata italic cyrillic",
    "Literata italic latin",
    "Literata normal cyrillic",
    "Literata normal latin",
  ]);
});

test("каждый файл шрифта подключён ровно одним @font-face со своего каталога", () => {
  const used = faces().map(({ file }) => file);

  assert.deepEqual(used.sort(), shipped());
});

test("текст OFL лежит рядом со шрифтами", () => {
  for (const family of ["literata", "golos-text"]) {
    const text = readFileSync(path.join(FONTS, `OFL-${family}.txt`), "utf8");
    assert.match(text, /SIL Open Font License, Version 1\.1/, `OFL-${family}.txt без текста лицензии`);
  }
});
