import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { readdir } from "node:fs/promises";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";

import { DIST, pages, route } from "../scripts/budget.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const INTERACTIVE = /<(a|button|input|textarea|select|summary)[\s>]/g;
const TOP = new Set(["/", "/ru/", "/en/"]);

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

async function bundled(extension) {
  const found = [];
  for (const entry of await readdir(DIST, { recursive: true, withFileTypes: true })) {
    if (entry.isFile() && entry.name.endsWith(extension)) {
      found.push(readFileSync(path.join(entry.parentPath, entry.name), "utf8"));
    }
  }
  return found;
}

async function styles() {
  const inline = (await bundled(".html")).flatMap((html) => [...html.matchAll(/<style>([\s\S]*?)<\/style>/g)].map(([, css]) => css));
  return [...(await bundled(".css")), ...inline].join("\n");
}

function focusable(css) {
  const covered = new Set();
  for (const [, selector] of css.matchAll(/([^{}]*):focus-visible[^{]*\{/g)) {
    for (const [, tag] of selector.matchAll(/(?:^|[\s(,])([a-z]+)(?=[\s),]|$)/g)) covered.add(tag);
  }
  return covered;
}

test("каждый интерактивный элемент попадает под правило видимого фокуса", async () => {
  const covered = focusable(await styles());

  for (const file of await pages()) {
    const html = readFileSync(file, "utf8");
    for (const [, tag] of html.matchAll(INTERACTIVE)) {
      assert.ok(covered.has(tag), `${route(file)}: <${tag}> без видимого фокуса`);
    }
  }
});

test("порядок фокуса не переопределён положительным tabindex", async () => {
  for (const file of await pages()) {
    const html = readFileSync(file, "utf8");
    assert.doesNotMatch(html, /tabindex=["']?[1-9]/, `${route(file)} переставляет порядок фокуса`);
  }
});

test("ссылка на уровень вверх стоит перед содержимым экрана", async () => {
  for (const file of await pages()) {
    if (TOP.has(route(file))) continue;
    const html = readFileSync(file, "utf8");
    const back = html.indexOf("data-back");

    assert.notEqual(back, -1, `${route(file)} без ссылки на уровень вверх`);
    assert.ok(back < html.indexOf("<main"), `${route(file)}: ссылка назад идёт после содержимого`);
  }
});

test("обработчик клавиатуры уехал в сборку и подключён к каждой странице", async () => {
  const keys = (await bundled(".js")).filter((source) => source.includes("data-filter"));

  assert.ok(
    keys.some((source) => source.includes("Escape")),
    "в сборке нет обработчика Esc и слэша",
  );
  for (const file of await pages()) {
    if (route(file) === "/") continue;
    assert.match(readFileSync(file, "utf8"), /<script type="module"/, `${route(file)} без сценариев`);
  }
});
