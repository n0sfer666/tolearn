import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";

import { DIST } from "../scripts/budget.mjs";
import { early } from "../scripts/theme.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const PAGES = ["", ...["ru", "en"].flatMap((locale) => ["", "program", "topic", "exam", "review"].map((screen) => path.join(locale, screen)))];

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

function page(route) {
  return readFileSync(path.join(DIST, route, "index.html"), "utf8");
}

function storage(stored) {
  const store = new Map(stored === undefined ? [] : [["tolearn.theme", stored]]);
  return {
    store,
    api: {
      getItem: (key) => (store.has(key) ? store.get(key) : null),
      setItem: (key, value) => store.set(key, value),
      removeItem: (key) => store.delete(key),
    },
  };
}

function world(stored) {
  const root = { dataset: {} };
  const { store, api } = storage(stored);
  return { root, store, context: { document: { documentElement: root }, localStorage: api } };
}

function run(code, stored) {
  const open = world(stored);
  runInNewContext(code, open.context);
  return open;
}

test("выбор темы применяется в <head> до разметки — на каждой странице", () => {
  for (const route of PAGES) {
    const html = page(route);
    const head = html.slice(0, html.indexOf("<body"));
    assert.ok(head.includes("tolearn.theme"), `${route || "/"}: стартовый скрипт темы не в <head>`);
    assert.ok(!/<script[^>]*\bsrc=/.test(early(html)), `${route || "/"}: скрипт темы не встроенный`);
  }
});

test("сохранённая тема ставится атрибутом до первой отрисовки", () => {
  assert.equal(run(early(page("ru")), "dark").root.dataset.theme, "dark");
  assert.equal(run(early(page("ru")), "light").root.dataset.theme, "light");
});

test("системная тема не ставит атрибут — решает color-scheme", () => {
  assert.equal(run(early(page("ru")), undefined).root.dataset.theme, undefined);
  assert.equal(run(early(page("ru")), "sepia").root.dataset.theme, undefined);
});

test("сохранённый выбор переживает перезапуск на любом экране", () => {
  for (const route of PAGES) {
    const { root } = run(early(page(route)), "dark");
    assert.equal(root.dataset.theme, "dark", `${route || "/"}: после перезапуска тема сброшена`);
  }
});

test("переключатель темы живёт только в настройках", () => {
  for (const route of PAGES) {
    assert.doesNotMatch(page(route), /data-theme-choice/, `${route || "/"}: переключатель темы вне настроек`);
  }
});

test("тема не мешает языку: у страницы два независимых ключа", () => {
  const html = page("ru/topic");
  assert.ok(html.includes("tolearn.locale"));
  assert.ok(html.includes("tolearn.theme"));
});
